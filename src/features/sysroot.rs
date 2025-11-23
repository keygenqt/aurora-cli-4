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
    pub fn upgrade() -> Result<(), Box<dyn Error>> {
        let sysroot = &path::get_relative_path(&PathBuf::from(constants::SYSROOT_GLOBAL_DIR_PATH));
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
        // Create unix sysroot
        let sysroot_unix = if cfg!(windows) {
            match Confirm::new(&format!("Create unix sysroot for cross-compilation?"))
                .with_default(false)
                .prompt()
            {
                Ok(result) => {
                    if result {
                        let sysroot_unix = &sysroot.parent().unwrap().join(constants::SYSROOT_UNIX_DIR_NAME);
                        if sysroot_unix.exists() {
                            fs::remove_dir_all(sysroot_unix)?;
                        }
                        Some(&sysroot_unix.clone())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        };
        // Init docker
        let docker_image = match Docker::get_image() {
            Ok(image) => image,
            Err(err) => crate::exit_error!("{}", err),
        };
        let docker = &Docker::new_sys(docker_image);
        // Up container
        docker.run_container(&sysroot.parent().unwrap().to_path_buf())?;
        // Catch exit
        let docker_move = docker.clone();
        let _ = ctrlc::set_handler(move || {
            docker_move.stop_container();
            exit_info!("work has stopped")
        });
        // Run build
        let result = docker.copy_sysroots(sysroot, sysroot_unix);
        // Stop container
        docker.stop_container();
        // Convert symbolic link for windows
        #[cfg(windows)]
        {
            let result = Self::convert_symlink(&sysroot);
            if result.is_err() {
                fs::remove_dir_all(sysroot)?;
                Err("failed convert symlink, your enable Developer Mode?")?;
            }
        }
        // Result
        result
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
            match crate::tools::Exec::command(bash, ["-c", &format!("find {path_unix} -type l")]) {
                Ok(output) => {
                    for path_link_unix in String::from_utf8(output.stdout).unwrap().split("\n") {
                        let result =
                            crate::tools::Exec::command(bash, ["-c", &format!("readlink -f {path_link_unix}")])
                                .unwrap();
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
        crate::print_success!("symlink convert successfully");
        Ok(())
    }
}
