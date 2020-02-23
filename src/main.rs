use std::fs;

fn main()
{
    get_all_current_processes();
	let docker_id = get_docker_id_by_pid(&"7764".to_string());
    println!("id of the docker: {}", docker_id.unwrap());
}

fn get_all_current_processes()
{

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

	if !contents.contains("/docker")
	{
		return Err("Given pid is not a docker process".to_string());
	}

	let splitted: Vec<&str> = contents.split("\n").collect();
	let docker_id: &str = splitted[0].split("/").collect::<Vec<&str>>()[2];

	Ok(docker_id.to_string())
}
