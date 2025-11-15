use std::error::Error;
use std::fs;
use std::path::PathBuf;

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
                #[cfg(windows)]
                {
                    match Self::convert_symlink(&sysroot) {
                        Ok(_) => docker.stop_container(),
                        Err(err) => {
                            fs::remove_dir_all(sysroot)?;
                            docker.stop_container();
                            crate::print_error!("{}", err);
                            crate::exit_error!("failed convert symlink, your enable Developer Mode?");
                        }
                    }
                }
                #[cfg(unix)]
                {
                    docker.stop_container();
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

    #[cfg(windows)]
    fn convert_symlink(sysroot: &PathBuf) -> Result<(), Box<dyn Error>> {
        use crate::print_state;
        use crate::tools::arch::Arch;
        use std::os::windows::fs as winfs;
        for arch in Arch::entries() {
            print_state!("start convert symlink for arch '{}'", arch);
            let bash = "C:\\Program Files\\Git\\bin\\bash.exe";
            let path = sysroot.join(arch).to_string_lossy().to_string();
            let path_unix = path::windows_convert_to_unix(&path);
            match crate::tools::exec::exec_wait(bash, ["-c", &format!("find {path_unix} -type l")]) {
                Ok(output) => {
                    for path_link_unix in String::from_utf8(output.stdout).unwrap().split("\n") {
                        let result = crate::tools::exec::exec_wait(bash, ["-c", &format!("readlink -f {path_link_unix}")]).unwrap();
                        let path_file_owner = String::from_utf8(result.stdout).unwrap();
                        let path_file_owner = path_file_owner.trim();
                        if path_file_owner.is_empty() {
                            continue;
                        }
                        // Path from -> to
                        let path_link = PathBuf::from(path::windows_convert_from_unix(path_link_unix));
                        let path_orig = PathBuf::from(path::windows_convert_from_unix(path_file_owner));
                        // Create windows symlink
                        if path_link.is_file() || path_link.is_symlink() {
                            match fs::remove_file(&path_link) {
                                Ok(_) => {
                                    if path_orig.exists() {
                                        if path_orig.is_file() {
                                            winfs::symlink_file(&path_orig, &path_link)?;
                                        } else {
                                            winfs::symlink_dir(&path_orig, &path_link)?;
                                        }
                                    }
                                }
                                Err(_) => { /* if file not delete - skip */ }
                            }
                        }
                    }
                }
                Err(_) => Err("fail execute find symlink. Your install git: https://git-scm.com/install/windows?")?,
            }
        }
        Ok(())
    }
}
