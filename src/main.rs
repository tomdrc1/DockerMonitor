mod docker_helper;

use std::fs;
use std::process::Command;
use std::str;
use std::io::Read;

fn main()
{
	let docker = docker_helper::DockerHelper::new();

	let ids = docker.get_containers_ids();

	for id in ids
	{
		let image = docker.get_container_image(&id);
		
		docker.read_docker_image_and_get_hashs(&image);
	}
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