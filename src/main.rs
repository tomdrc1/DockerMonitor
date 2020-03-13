mod docker_helper;

fn main()
{
	let docker = docker_helper::DockerHelper::new("db.sqlite".to_string());

	loop
	{
		let ids = docker.get_containers_ids();
		for id in ids
		{
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
			
			if docker.is_valid_process(&id, &pid)
			{
				continue
			}
			println!("{}, {}", id, pid);
			println!("BAD PROCESS!!!!!");
			docker.restart_container(&id);
		}
	}
}