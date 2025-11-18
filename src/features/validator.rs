use std::error::Error;
use std::path::PathBuf;

use crate::exit_error;
use crate::exit_info;
use crate::tools::docker::Docker;
use crate::tools::path;

pub struct Validator {}

impl Validator {
    pub fn validate(path: PathBuf) -> Result<String, Box<dyn Error>> {
        let path = path::get_relative_path_canonicalize(&path);
        if path.is_none() {
            exit_error!("file not found")
        }
        let path = &path.unwrap();
        let path_dir = &path.parent().unwrap().to_path_buf();
        // Init docker
        let docker = &Docker::new(None);
        // Up container
        docker.run_container(path_dir);
        // Catch exit
        let docker_move = docker.clone();
        let _ = ctrlc::set_handler(move || {
            docker_move.stop_container();
            exit_info!("work has stopped")
        });
        // Moc files
        match docker.rpm_validate(&path) {
            Ok(output) => {
                docker.stop_container();
                Ok(output)
            },
            Err(err) => {
                docker.stop_container();
                Err(err)?
            }
        }
    }
}
