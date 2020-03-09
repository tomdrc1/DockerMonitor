mod db_helper;

extern crate tar;
extern crate shiplift;
extern crate tokio;
extern crate walkdir;
extern crate md5;

use walkdir::WalkDir;
use tar::Archive;
use serde_json;
use std::fs;
use std::{fs::OpenOptions, io::Write};
use tokio::prelude::{Future, Stream};
use std::io::Read;

const ELF_BYTES: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

pub struct DockerHelper
{
    connection: shiplift::Docker,
    db: db_helper::DBHelper
}

impl DockerHelper
{
    pub fn new() -> DockerHelper
    {
        let db = db_helper::DBHelper::new("db.sqlite".to_string());
        
        DockerHelper{connection: shiplift::Docker::new(), db: db}
    }

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
    /// * `image_name` - The name of the image we want to read and get the hash values of all the executables 
    pub fn read_docker_image_and_get_hashs(&self, image_name: &String)
    {
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
                    self.db.insert_file(&elf[4..].to_string(), &image_name, &hash);
                }
            }
            fs::remove_dir_all("out").unwrap();
        }
        fs::remove_dir_all("unpack").unwrap();
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

            //println!("{}", e.path().to_str().unwrap());
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
        let mut file = match fs::File::open(path)
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
