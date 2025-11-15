use std::error::Error;
use std::fs;
use std::path::PathBuf;

use crate::exit_error;
use crate::exit_info;
use crate::print_info;
use crate::print_warning;
use crate::tools::constants;
use crate::tools::docker::Docker;
use crate::tools::path;
use ctrlc;
use inquire::Confirm;

pub struct Sysroot {}

impl Sysroot {
    pub fn upgrade() -> Result<PathBuf, Box<dyn Error>> {
        let sysroot = &path::get_relative_path(&PathBuf::from(constants::SYSROOT_DIR_GLOBAL));
        // Remove dir if exist
        if sysroot.exists() {
            print_warning!("data found!");
            match Confirm::new(&format!("Delete {}", sysroot.to_string_lossy()))
                .with_default(false)
                .prompt()
            {
                Ok(result) => {
                    if result {
                        print_info!("remove data: {}", sysroot.to_string_lossy());
                        fs::remove_dir_all(sysroot)?
                    } else {
                        exit_info!("work has stopped")
                    }
                }
                Err(_) => exit_info!("work has stopped"),
            }
        }
        // Create folder for sysroot
        fs::create_dir_all(sysroot)?;
        // Init docker
        let docker = &Docker::new(Some(constants::CONTAINER_SYS_PREFIX.to_string()));
        // Up container
        docker.run_container(&sysroot);
        // Catch exit
        let docker_move = docker.clone();
        let _ = ctrlc::set_handler(move || {
            docker_move.stop_container();
            exit_info!("work has stopped")
        });
        // Copy sysroot
        match docker.copy_sysroots() {
            Ok(_) => {
                // #[cfg(windows)]
                let result = Self::convert_symlink(&sysroot);
                // Stop container
                docker.stop_container();
                // Check result
                if result.is_err() {
                    fs::remove_dir_all(sysroot)?;
                    exit_error!("failed convert symlink, your enable Developer Mode?");
                }
            }
            Err(_) => {
                // Stop container
                docker.stop_container()
            }
        }
        // Done
        Ok(sysroot.to_path_buf())
    }

    fn convert_symlink(_: &PathBuf) -> Result<(), Box<dyn Error>> {
        // @todo add convert symlink for windows
        Ok(())
    }
}
