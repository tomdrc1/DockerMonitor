mod docker_helper;
mod db_helper;

extern crate walkdir;

use walkdir::WalkDir;
use std::fs;
use std::process::Command;
use std::str;
use std::io::Read;

const ELF_BYTES: [u8; 4] = [0x7f, 0x45, 0x4c, 0x46];

fn main()
{
	let mut docker = docker_helper::DockerHelper::new();
	let db = db_helper::DBHelper::new("db.sqlite".to_string());

	db.create_tables();

	let image = docker.get_container_image("c99e075851dbac4ccef5c26a73046214ec138c60202bcc0235fe81a66c79c7ba".to_string());
	
	docker.read_docker_image(&image);	
	let elfs = get_all_elfs_in_dir("out");
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
fn get_all_current_processes() -> Vec<String>
{
	let process = Command::new("ps").args(&["-A", "-o", "pid"]).output().expect("process failed to execute");

	let processes = str::from_utf8(&process.stdout).unwrap();
	let processes_split: Vec<&str> = processes.split('\n').collect();

	let mut clean_processes: Vec<String> = Vec::new();

	for process in processes_split
	{
		let cleaned_process = process.trim();

		clean_processes.push(cleaned_process.to_string());
	}

	clean_processes
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
fn get_docker_id_by_pid(pid: &String) -> std::result::Result<String, String>
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

	let docker_id: &str = splitted[0].split("/").collect::<Vec<&str>>()[2];

	Ok(docker_id.to_string())
}

/// Returns a vector with all the elfs of root dir. Checking recursivly
/// 
/// # Arguments 
/// * `dir_name` - The top dir to start checking from
fn get_all_elfs_in_dir(dir_name: &str) -> Vec<String>
{
	let mut elfs: Vec<String> = Vec::new();

	for entry in WalkDir::new(dir_name) 
	{
		let e = entry.unwrap();
		if is_elf(&e.path().to_str().unwrap().to_string())
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
fn is_elf(path: &String) -> bool
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