mod db_helper;

extern crate tar;
extern crate shiplift;
extern crate tokio;
extern crate walkdir;
extern crate md5;
extern crate sysinfo;

use walkdir::WalkDir;
use tar::Archive;
use serde_json;
use std::fs;
use std::{fs::OpenOptions, io::Write};
use tokio::prelude::{Future, Stream};
use std::io::Read;
use std::str;
use sysinfo::SystemExt;

const ELF_BYTES: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

pub struct DockerHelper
{
    connection: shiplift::Docker,
    db: db_helper::DBHelper
}

impl DockerHelper
{
    /// Will make a new docker helper. Running this will also generate a new DBHelper that will make a new db
    /// 
    /// # Arguments
    /// * `db_file_name` - The name of the db file.
    pub fn new(db_file_name: String) -> DockerHelper
    {
        let db = db_helper::DBHelper::new(db_file_name);
        
        DockerHelper{connection: shiplift::Docker::new(), db: db}
    }

    /// Will return all the containers ids that are currently running in the system
    pub fn get_containers_ids(&self) -> Vec<String>
    {
        let mut runtime = tokio::runtime::Runtime::new().expect("Couldn't make runtime");
        let containers = runtime.block_on(self.connection.containers().list(&Default::default())).unwrap();

        let mut ids: Vec<String> = Vec::new();

        for container in containers
        {
            ids.push(container.id);
        }

        ids
    }

    /// Will return the image digest from the container id
    /// 
    /// # Arguments
    /// * `id` - The id of the container we want to get the image from
    pub fn get_container_image(&self, id: &String) -> String
    {
        let mut runtime = tokio::runtime::Runtime::new().expect("Couldn't make runtime");
        let container = runtime.block_on(self.connection.containers().get(&id).inspect()).unwrap();

        container.image
    }

    /// Will read a docker image and will put all the files values into the db
    /// 
    /// # Arguments
    /// * `image_name` - The name (or digest) of the image we want to read and get the hash values of all the executables 
    pub fn read_docker_image_and_get_hashs(&self, image_name: &String)
    {
        if self.db.is_image_hashed(&image_name) == 1
        {
            return;
        }
        println!("Found new container, reading image {}", image_name);

        let mut export_file = OpenOptions::new().write(true).create(true).open("saved_image").unwrap();

        let fut = self.connection.images().get(image_name).export().for_each(move |bytes| {
            export_file.write(&bytes[..]).map(|_| (())).map_err(shiplift::errors::Error::IO)

        }).map_err(|e| eprintln!("Error: {}", e));

        tokio::run(fut);

        let mut tar_archive = Archive::new(fs::File::open("saved_image").unwrap());
        tar_archive.unpack("unpack").unwrap();
        fs::remove_file("saved_image").unwrap();

        let manifest_data = fs::read_to_string("unpack/manifest.json").unwrap();
        let manifest: serde_json::Value = match serde_json::from_str(&manifest_data) 
        {
            Ok(json_data) => json_data,
            Err(_) => serde_json::Value::Null
        };

        let layers = manifest[0]["Layers"].as_array().unwrap();

        for layer in layers
        {
            let mut layer_tar = Archive::new(fs::File::open(format!("unpack/{}", layer.as_str().unwrap())).unwrap());
            
            match layer_tar.unpack("out")
            {
                Ok(_) => (()),
                Err(_) => (())
            };

            let paths = fs::read_dir("out").unwrap();

            for path in paths
            {
                for elf in self.get_all_elfs_in_dir(path.unwrap().path().to_str().unwrap())
                {
                    let hash = match self.get_hash_of_file(&elf)
                    {
                        Ok(hash) => hash,
                        Err(_) => continue
                    };
                    // Slicing starting from index 3 to remove the 'out' folder from the path
                    self.db.insert_file(&elf[3..].to_string(), &image_name, &hash);
                }
            }
            fs::remove_dir_all("out").unwrap();
        }
        
        fs::remove_dir_all("unpack").unwrap();
        self.db.insert_image(image_name, 1);
        println!("Finished reading image {}", image_name);
    }

    /// Will check if the process is a valid container process, this is done by checking the hash from the db of the image and comparing it to the hash we will calculate now from the exe link
    /// Returns weather the process is valid or not
    /// 
    /// # Arguments
    /// * `container_id` - The id of the container we want to get the image digest of
    /// * `pid` - The process id of the process we want to check is valid in the container. 
    pub fn is_valid_process(&self, container_id: &String, pid: &String) -> bool
    {
        let digest = self.get_container_image(&container_id);
        let path = self.get_process_path_by_pid(pid);
        self.read_docker_image_and_get_hashs(&digest); // Calling this to make sure we have the hashes. If not make it
        let correct_hash = self.db.get_hash(&path, &digest);

        if correct_hash.is_empty()
        {
            println!("File is not in the image data!");
            return false;
        }

        let hash = match self.get_hash_of_file(&format!("/proc/{}/exe", pid))
        {
            Ok(hash) => hash,
            Err(_) => return false
        };

        if correct_hash != hash
        {
            println!("The file hash was not the same!");
            return false;
        }

        true
    }

    /// Returns a vector of all the current processes ids.
    /// 
    /// # Examples
    /// ```
    /// fn main()
    /// {
    /// 	let processes = get_all_current_processes()
    /// 
    ///		for process in processes 
    /// 	{
    ///			match get_docker_id_by_pid(&process.trim().to_string()) 
    ///			{
    ///				Ok(id) => println!("process {} is a process of docker {}", process, id),
    ///				Err(_) => print!("Err: {} is not a docker process!", process)
    ///			};
    /// 	}
    /// }
    /// ```
    pub fn get_all_current_processes(&self) -> Vec<String>
    {
        let sys = sysinfo::System::new_all();
        let mut pids: Vec<String> = Vec::new();

        for (pid, _) in sys.get_processes() {
            pids.push(pid.to_string());
        }
        
        pids
    }
    
    /// Returns the docker id of the given process
    /// 
    /// # Arguments
    /// * `pid` - The process id that we want to check if it's a docker process, if it is return the docker id
    /// 
    /// # Errors
    /// * `Error opening file!` - An error returned if you gave a wrong process id or for some unkown reason you don't have the cgroup file.
    /// * `Given pid is not a docker process` - An error returned if the given pid is not a docker prcoess.
    /// 
    /// # Examples
    /// ```
    /// fn main()
    /// {
    /// 	if is_process_docker("10".to_string()).unwrap()
    /// 	{
    /// 		println!("The process with the pid 10 is a docker process!");
    /// 	}
    /// }
    /// ```
    pub fn get_container_id_by_pid(&self, pid: &String) -> std::result::Result<String, String>
    {
        let contents = match fs::read_to_string(format!("/proc/{}/cgroup", pid)) 
        {
            Ok(file_data) => file_data,
            Err(_) => {
                return Err("Error opening file!".to_string());
            }
        };

        let splitted: Vec<&str> = contents.split("\n").collect();
        let is_docker: &str = splitted[0].split("/").collect::<Vec<&str>>()[1];

        if is_docker != "docker"
        {
            return Err("Given pid is not a docker process".to_string());
        }

        let container_id: &str = splitted[0].split("/").collect::<Vec<&str>>()[2];
        Ok(container_id.to_string())
    }

    pub fn restart_container(&self, container_id: &String)
    {
        let image = self.get_container_image(&container_id);
        let mut runtime = tokio::runtime::Runtime::new().expect("Couldn't make runtime");
        let container = self.connection.containers().get(container_id);

        runtime.block_on(container.kill(Some(""))).unwrap();
        runtime.block_on(container.remove(shiplift::RmContainerOptions::default())).unwrap();

        runtime.block_on(self.connection.containers().create(&shiplift::ContainerOptions::builder(image.as_ref()).build())).unwrap();
    }

    /// Returns the process path by the pid
    /// 
    /// # Arguments
    /// * `pid` - The process id we want to get the path of
    fn get_process_path_by_pid(&self, pid: &String) -> String
    {
        let path = match fs::read_link(format!("/proc/{}/exe", pid))
        {
            Ok(path) => path,
            Err(_) => return String::new()
        };

        path.to_str().unwrap().to_string()
    }

    /// Returns a vector with all the elfs of root dir. Checking recursivly
    /// 
    /// # Arguments 
    /// * `dir_name` - The top dir to start checking from
    fn get_all_elfs_in_dir(&self, dir_name: &str) -> Vec<String>
    {
        let mut elfs: Vec<String> = Vec::new();

        for entry in WalkDir::new(dir_name) 
        {
            let e = match entry
            {
                Ok(e) => e,
                Err(_) => return Vec::new()
            };

            if self.is_elf(&e.path().to_str().unwrap().to_string())
            {
                elfs.push(e.path().to_str().unwrap().to_string());
            }
        }

        elfs
    }

    /// Returns weather the file on the given path is an elf or not
    /// 
    /// # Arguments
    /// * `path` - A string to the file
    fn is_elf(&self, path: &String) -> bool
    {
        if path.contains("/dev/")
        {
            return false;
        }

        let real_path = match fs::read_link(path)
        {
            Ok(real_path) => real_path.to_str().unwrap().to_string(),
            Err(_) => path.to_string()
        };

        if real_path.contains("/dev/")
        {
            return false;
        }

        let mut file = match fs::File::open(real_path)
        {
            Ok(file) => file,
            Err(_) => return false
        };
        
        let mut bytes: [u8; 4] = [0; 4];

        match file.read_exact(&mut bytes)
        {
            Ok(_) => (()),
            Err(_) => return false
        };

        bytes.iter().zip(ELF_BYTES.iter()).all(|(a,b)| a == b)
    }

    /// Returns the hash of the file in the given path. (md5)
    /// 
    /// # Arguments
    /// * `path` - The path to file we want to get the md5 hash of
    /// 
    /// # Errors
    /// * `Couldn't open file for reading!` - An error returned if you gave a wrong path.
    fn get_hash_of_file(&self, path: &String) -> std::result::Result<String, String>
    {
        let mut file = match fs::File::open(path)
        {
            Ok(file) => file,
            Err(_) => return Err("Couldn't open file for reading!".to_string())
        };

        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        let hash = md5::compute(buffer);

        Ok(format!("{:#?}", hash))
    }
}
