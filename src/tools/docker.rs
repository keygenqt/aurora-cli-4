use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use inquire::Select;
use regex::Regex;
use uuid::Uuid;

use crate::print_info;
use crate::print_state;
use crate::print_success;
use crate::tools::arch::Arch;
use crate::tools::constants;
use crate::tools::exec::Exec;
use crate::tools::path;

#[derive(Clone)]
pub struct Docker {
    image: String,
    container: String,
}

#[allow(dead_code)]
impl Docker {
    pub fn get_image() -> Result<String, Box<dyn Error>> {
        let images = Self::find_images()?;
        if images.is_empty() {
            Err("images build-tools not found")?
        }
        if images.len() == 1 {
            return Ok(images[0].clone());
        }
        let image = Select::new("Which build-tools image should I use?", images).prompt()?;
        Ok(image)
    }

    pub fn new(image: String) -> Self {
        Docker {
            image: image,
            container: format!("{}-{}", constants::CONTAINER_PREFIX, Uuid::new_v4().to_string()),
        }
    }

    pub fn new_sys(image: String) -> Self {
        Docker {
            image: image,
            container: format!("{}-{}", constants::CONTAINER_PREFIX_SYS, Uuid::new_v4().to_string()),
        }
    }

    pub fn run_container(&self, mount: &PathBuf) -> Result<(), Box<dyn Error>> {
        self.check_running_sys()?;
        let is_container_in_container = env::var("AURORA_CLI_CONTAINER").is_ok();
        let mode = if is_container_in_container { "-d" } else { "-itd" };

        let mount_path = path::windows_clear(&mount.to_string_lossy().to_string());
        let mount_path = path::get_resolve_host_path(&mount_path);
        let mount_arg = format!("{}:/project", mount_path);

        let mut args: Vec<&str> = vec![
            "run",
            "--rm",
            "--name",
            &self.container,
            "-v",
            &mount_arg,
            mode,
            &self.image,
        ];

        if is_container_in_container {
            args.push("sleep");
            args.push("infinity");
        }

        let result = Exec::command("docker", &args);
        if result.is_err() {
            Err(format!("failed to run container from image '{}'", &self.image))?
        }

        Ok(())
    }

    pub fn stop_container(&self) {
        let _ = Exec::command("docker", ["stop", &self.container]);
    }

    pub fn moc_files(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let prepared_path = path::windows_clear(&path.to_string_lossy().to_string());
        let search: Vec<String> = path::search_files_by_ext(&PathBuf::from(&prepared_path), "h");
        for path in search {
            let path = path::windows_clear(&path);
            if path.contains("/target/") || path.contains("\\target\\") {
                continue;
            }
            if !fs::read_to_string(&path)?.contains("Q_OBJECT") {
                continue;
            }
            print_state!("moc file: {}", path);
            let file_docker = if cfg!(windows) {
                path::windows_convert_to_unix(&path.replace(&prepared_path, "/project"))
            } else {
                path.replace(&prepared_path, "/project")
            };

            let path = PathBuf::from(file_docker);
            let file_dir = path.parent().unwrap().to_string_lossy().to_string();
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let file_name_moc = file_name.replace(".h", ".moc");

            let command = format!("cd {file_dir} && moc {file_name} > {file_name_moc} && chmod -R 777 {file_name_moc}");
            Exec::command_verbose("docker", ["exec", &self.container, "/bin/bash", "-c", &command])?;
        }
        Ok(())
    }

    pub fn rpm_build(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let prepared_path = path::windows_clear(&path.to_string_lossy().to_string());
        // Get spec file
        let search: Vec<String> = path::search_files_by_ext(&PathBuf::from(&prepared_path), "spec");
        if search.is_empty() {
            Err("spec file not found")?
        }
        // Get path spec docker
        let file_spec = search.iter().next().unwrap();
        let file_spec_docker = if cfg!(windows) {
            path::windows_convert_to_unix(&file_spec.replace(&prepared_path, "/project"))
        } else {
            file_spec.replace(&prepared_path, "/project")
        };
        // Get arch
        let path_bin_dir = PathBuf::from(&prepared_path).join("bin");
        if !path_bin_dir.exists() || !path_bin_dir.is_dir() {
            Err("create a bin folder with the executable file: <project> -> bin -> executable")?
        }
        let arch = match path_bin_dir.read_dir() {
            Ok(entries) => {
                let mut arch: Option<String> = None;
                for entry_result in entries {
                    if let Ok(entry) = entry_result {
                        let path = entry.path();
                        if path.is_file() {
                            // Get path file docker
                            let file_bin = path.to_string_lossy();
                            let file_bin_docker = if cfg!(windows) {
                                path::windows_convert_to_unix(&file_bin.replace(&prepared_path, "/project"))
                            } else {
                                file_bin.replace(&prepared_path, "/project")
                            };
                            // Get arch
                            let command = format!("file {file_bin_docker}");
                            match Exec::command("docker", ["exec", &self.container, "/bin/bash", "-c", &command]) {
                                Ok(result) => {
                                    if result.contains("ELF") {
                                        if result.contains("x86-64") {
                                            arch = Some(Arch::X86_64.name());
                                        }
                                        if result.contains("aarch64") {
                                            arch = Some(Arch::Aarch64.name());
                                        }
                                        if result.contains("arm") && result.contains("32-bit") {
                                            arch = Some(Arch::Armv7hl.name());
                                        }
                                        break;
                                    }
                                }
                                Err(_) => Err(format!("failed to retrieve file data: {}", path.to_string_lossy()))?,
                            }
                        }
                    }
                }
                arch
            }
            Err(_) => Err("in bin folder executable file not found")?,
        };
        if arch.is_none() {
            Err("arch binary file not found")?
        }
        let arch = arch.unwrap();
        // Fix \r in spec file
        if cfg!(windows) {
            Exec::command(
                "docker",
                [
                    "exec",
                    &self.container,
                    "/bin/bash",
                    "-c",
                    &format!("cd /project && find . -name *.spec -type f -exec sed -i 's/\r//g' {{}} +"),
                ],
            )?;
        };
        print_state!("start build package '{}'", arch);
        // Exec build
        let result = Exec::command_verbose("docker", [
            "exec",
            &self.container,
            "/bin/bash",
            "-c",
            &format!("cd /project && rpmbuild -ba --build-in-place --target '{arch}' --define '_topdir /project' {file_spec_docker} && chmod -R 777 /project"),
            ]);
        if result.is_err() {
            Err("RPM package packaging failed with an error")?;
        }
        // Clear temp files
        for file in vec![
            "BUILDROOT",
            "SRPMS",
            "debugfiles.list",
            "debuglinks.list",
            "debugsourcefiles.list",
            "debugsources.list",
            "documentation.list",
            "elfbins.list",
        ] {
            let path = PathBuf::from(prepared_path.clone()).join(file);
            if path.exists() {
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    pub fn rpm_sign(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let rpm_name = path.file_name().unwrap().to_string_lossy();
        let project = &path.parent().unwrap().to_path_buf();
        let keys = if cfg!(windows) {
            path::get_path_home().join(".auroraos-regular-keys")
        } else {
            path::get_path_home().join("AuroraOS").join("package-signing")
        };
        if !keys.exists() || !keys.is_dir() {
            Err(format!("keys directory not found: {}", keys.to_string_lossy()))?
        }
        let cert = keys.join("regular_cert.pem");
        if !cert.exists() || !cert.is_file() {
            Err(format!("cert not found: {}", cert.to_string_lossy()))?
        }
        let key = keys.join("regular_key.pem");
        if !key.exists() || !key.is_file() {
            Err(format!("key not found: {}", cert.to_string_lossy()))?
        }
        // Copy keys to project
        fs::copy(cert, project.join("regular_cert.pem"))?;
        fs::copy(key, project.join("regular_key.pem"))?;

        print_state!("start sign package '{}'", rpm_name);

        // Execute sing rpm package
        let command = format!("rpmsign-external sign --force --key=/project/regular_key.pem --cert=/project/regular_cert.pem /project/{rpm_name}");
        let result = Exec::command_verbose("docker", ["exec", &self.container, "/bin/bash", "-c", &command]);

        // Remove keys from project
        let _ = fs::remove_file(project.join("regular_cert.pem"));
        let _ = fs::remove_file(project.join("regular_key.pem"));

        // Result
        if result.is_err() {
            Err("failed to sign the package")?
        }
        Ok(())
    }

    pub fn copy_sysroots(&self, sysroot: &PathBuf, sysroot_unix: Option<&PathBuf>) -> Result<(), Box<dyn Error>> {
        let paths = match Exec::command("docker", ["exec", &self.container, "/bin/bash", "-c", "ls /opt/cross"]) {
            Ok(output) => output
                .stdout
                .iter()
                .map(|e| e.to_string())
                .filter(|e| {
                    for arch in Arch::entries() {
                        if e.contains(&arch) {
                            return true;
                        }
                    }
                    false
                })
                .collect::<Vec<String>>(),
            Err(_) => Err("failed get sysroot directories")?,
        };
        // Copy
        for sysroot_path in vec![Some(sysroot), sysroot_unix] {
            if let Some(sysroot_path) = sysroot_path {
                let sysroot_name = sysroot_path.file_name().unwrap().to_str().unwrap();
                print_info!(
                    "copy sysroot from '{}' to {}",
                    self.image,
                    sysroot_path.to_string_lossy()
                );
                for arch in Arch::entries() {
                    let path_sysroot = format!("/project/{}/{}", sysroot_name, arch);
                    let path = paths.iter().filter(|e| e.contains(&arch)).next();
                    for folder_name in ["lib", "include"] {
                        let path_container = if let Some(path) = path {
                            format!("/opt/cross/{}/sys-root/usr/{}", path, folder_name)
                        } else {
                            format!("/usr/{}", folder_name)
                        };
                        let command = format!(
                            "mkdir -p {path_sysroot}/usr && cp -a {path_container} {path_sysroot}/usr && chmod -R 777 /project/{sysroot_name}",
                        );
                        print_state!("start copy {} for arch '{}'", folder_name, arch);
                        let _ = Exec::command("docker", ["exec", &self.container, "/bin/bash", "-c", &command]);
                    }
                }
                print_success!("sysroot successfully created: {}", sysroot_path.to_string_lossy());
            }
        }
        Ok(())
    }

    pub fn rpm_validate(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let rpm_docker = format!("/project/{}", path.file_name().unwrap().to_string_lossy());
        let command = format!("rpm-validator -p regular {rpm_docker}");
        let result = Exec::command_verbose("docker", ["exec", &self.container, "/bin/bash", "-c", &command])?;
        if result.contains("(ERROR)") {
            Err("errors were found during validation")?
        }
        Ok(())
    }

    fn find_images() -> Result<Vec<String>, Box<dyn Error>> {
        let image_env = env::var("AURORA_CLI_IMAGE");
        if image_env.is_ok() {
            return Ok(vec![image_env.unwrap()]);
        }
        match Exec::command("docker", ["images", "--format", "{{.Repository}}:{{.Tag}}"]) {
            Ok(output) => Ok(output
                .stdout
                .iter()
                .filter(|e| {
                    constants::BT_IMAGES_SEARCH
                        .iter()
                        .any(|r| Regex::new(r).unwrap().is_match(e))
                })
                .map(|e| e.to_string())
                .collect::<Vec<String>>()),
            Err(_) => Err("failed search images")?,
        }
    }

    fn check_running_sys(&self) -> Result<(), Box<dyn Error>> {
        if self.container.contains(constants::CONTAINER_PREFIX_SYS) {
            let result = Exec::command("docker", ["ps", "--format", "{{.Names}}"])?;
            let container = result
                .stdout
                .iter()
                .find(|e| e.contains(constants::CONTAINER_PREFIX_SYS));
            if let Some(container) = container {
                Err(format!("sysroot creation process is already in progress, please wait until it completes, or stop container: {}", container))?
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
