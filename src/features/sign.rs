use std::error::Error;
use std::path::PathBuf;

use crate::exit_error;
use crate::exit_info;
use crate::tools::docker::Docker;
use crate::tools::path;

pub struct RpmSign {}

impl RpmSign {
    pub fn sign(path: PathBuf) -> Result<(), Box<dyn Error>> {
        let path = path::get_relative_path_canonicalize(&path);
        if path.is_none() {
            exit_error!("file not found")
        }
        let path = path.unwrap();
        // Init docker
        let docker_image = match Docker::get_image() {
            Ok(image) => image,
            Err(err) => exit_error!("{}", err),
        };
        let docker = &Docker::new(docker_image);
        // Up container
        docker.run_container(&path.parent().unwrap().to_path_buf())?;
        // Catch exit
        let docker_move = docker.clone();
        let _ = ctrlc::set_handler(move || {
            docker_move.stop_container();
            exit_info!("work has stopped")
        });
        // Run build
        let result = docker.rpm_sign(&path);
        // Stop container
        docker.stop_container();
        // Result
        result
    }
}
