mod docker_helper;

fn main()
{
	let docker = docker_helper::DockerHelper::new("db.sqlite".to_string());
	let self_container_id = match docker.get_container_id_by_pid(&"self".to_string())
	{
		Ok(id) => id,
		Err(_) => "".to_string()
	};
	
	println!("Started container monitoring!");
	loop
	{
		let ids = docker.get_containers_ids();
		for id in ids
		{
			if self_container_id == id
			{
				continue
			}
			let image = docker.get_container_image(&id);
			
			docker.read_docker_image_and_get_hashs(&image);
		}
		
		let pids = docker.get_all_current_processes();
		
		for pid in pids
		{
			let id = match docker.get_container_id_by_pid(&pid)
			{
				Ok(id) => id,
				Err(_) => continue
			};
			
			if self_container_id == id || docker.is_valid_process(&id, &pid)
			{
				continue
			}
			println!("Container: {} Running image: {} had a bad process with the pid of: {}", id, docker.get_container_image(&id), pid);

			docker.restart_container(&id);
		}
	}
}