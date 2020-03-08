extern crate tar;
extern crate shiplift;
extern crate tokio;

use tar::Archive;
use serde_json;
use std::fs;
use std::{fs::OpenOptions, io::Write};
use tokio::prelude::{Future, Stream};

pub struct DockerHelper
{
    connection: shiplift::Docker
}

impl DockerHelper
{
    pub fn new() -> DockerHelper
    {
        DockerHelper{connection: shiplift::Docker::new()}
    }

    /// Will return the image digest from the container id
    /// 
    /// # Arguments
    /// * `id` - The id of the container we want to get the image from
    pub fn get_container_image(&self, id: String) -> String
    {
        let mut runtime = tokio::runtime::Runtime::new().expect("Couldn't make runtime");
        let container = runtime.block_on(self.connection.containers().get(&id).inspect()).unwrap();

        container.image
    }

    /// Will read the given docker image and will put all the docker's image files into a folder names out
    /// 
    /// # Arguments
    /// * `image_name` - The name of the image we want to read and get the hash values of all the executables 
    pub fn read_docker_image(&self, image_name: &String)
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
        }

        fs::remove_dir_all("unpack").unwrap();
    }
}
