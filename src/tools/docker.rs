use std::error::Error;
use std::fs;
use std::path::PathBuf;

use rust_search::SearchBuilder;
use uuid::Uuid;

use crate::exit_error;
use crate::exit_info;
use crate::print_info;
use crate::print_state;
use crate::print_warning;
use crate::tools::arch::Arch;
use crate::tools::constants;
use crate::tools::exec;
use crate::tools::path;

#[derive(Clone)]
pub struct Docker {
    image: String,
    container: String,
}

impl Docker {
    pub fn new(prefix: Option<String>) -> Self {
        fn find_build_tools() -> String {
            match exec::exec_wait("docker", ["images", "--format", "{{.Repository}}:{{.Tag}}"]) {
                Ok(output) => {
                    let out = String::from_utf8(output.stdout)
                        .unwrap()
                        .split("\n")
                        .filter(|e| e.contains(constants::BT_IMAGE_PREFIX))
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>();
                    match out.len() {
                        0 => exit_error!("not found image {}*", constants::BT_IMAGE_PREFIX),
                        1 => out.iter().next().unwrap().clone(),
                        _ => {
                            print_warning!("found multiple image, select first");
                            out.iter().next().unwrap().clone()
                        }
                    }
                }
                Err(_) => exit_error!("failed to execute docker command"),
            }
        }
        Docker {
            image: find_build_tools(),
            container: match prefix {
                Some(prefix) => format!("{}-{}", prefix, Uuid::new_v4().to_string()),
                None => format!("{}-{}", constants::CONTAINER_PREFIX, Uuid::new_v4().to_string()),
            },
        }
    }

    pub fn is_running() -> bool {
        match exec::exec_wait("docker", ["ps", "--format", "{{.Names}}"]) {
            Ok(output) => String::from_utf8(output.stdout)
                .unwrap()
                .split("\n")
                .any(|e| e.contains(constants::CONTAINER_SYS_PREFIX)),
            Err(_) => exit_error!("failed docker execute"),
        }
    }

    pub fn run_container(&self, mount: &PathBuf) {
        if Self::is_running() {
            exit_info!("sysroot creation process is already in progress, please wait until it completes, or stop container: {}", self.container);
        }
        match exec::exec_wait(
            "docker",
            [
                "run",
                "--rm",
                "--name",
                &self.container,
                "-v",
                &format!("{}:/project", mount.to_string_lossy()),
                "-itd",
                &self.image,
            ],
        ) {
            Ok(_) => print_info!("run container '{}'", &self.container),
            Err(_) => exit_error!("failed to run container"),
        }
    }

    pub fn stop_container(&self) {
        match exec::exec_wait("docker", ["stop", "container", &self.container]) {
            Ok(_) => print_info!("stop container '{}'", &self.container),
            Err(_) => exit_error!("failed stop container"),
        }
    }

    pub fn copy_sysroots(&self) -> Result<(), Box<dyn Error>> {
        let paths = match exec::exec_wait("docker", ["exec", &self.container, "/bin/bash", "-c", "ls /opt/cross"]) {
            Ok(output) => String::from_utf8(output.stdout)
                .unwrap()
                .split("\n")
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
        for arch in Arch::entries() {
            let path_sysroot = format!("/project/{}", arch);
            let path = paths.iter().filter(|e| e.contains(&arch)).next();
            for folder_name in ["lib", "lib64", "include"] {
                let path_container = if let Some(path) = path {
                    format!("/opt/cross/{}/sys-root/usr/{}", path, folder_name)
                } else {
                    format!("/usr/{}", folder_name)
                };
                let command = format!(
                    r#"mkdir -p {path_sysroot}/usr && cp -a {path_container} {path_sysroot}/usr && chmod -R 777 /project"#,
                );
                print_info!("start copy sysroot {} for arch '{}'", folder_name, arch);
                match exec::exec_wait("docker", ["exec", &self.container, "/bin/bash", "-c", &command]) {
                    Ok(output) => {
                        for line in  String::from_utf8(output.stdout).unwrap().split("\n") {
                            println!("{}", line);
                        }
                    },
                    Err(_) => {},
                }
            }
        }
        Ok(())
    }

    pub fn moc_files(&self, project: &PathBuf) -> Result<(), Box<dyn Error>> {
        let search: Vec<String> = SearchBuilder::default()
            .location(project)
            .ext("h")
            .strict()
            .ignore_case()
            .build()
            .collect();
        for path in search {
            if path.contains("/target/") {
                continue;
            }
            if !fs::read_to_string(&path)?.contains("Q_OBJECT") {
                continue;
            }
            print_state!("moc file: {}", path);
            let file_docker = path.replace(&project.to_string_lossy().to_string(), "/project");

            let path = PathBuf::from(file_docker);
            let file_dir = path.parent().unwrap().to_string_lossy().to_string();
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();
            let file_name_moc = file_name.replace(".h", ".moc");

            let command = format!(r#"cd {file_dir} && moc {file_name} > {file_name_moc} && chmod -R 777 {file_name_moc}"#);
            exec::exec_wait("docker", ["exec", &self.container, "/bin/bash", "-c", &command])?;
        }
        Ok(())
    }

    pub fn rpm_build(&self, project: &PathBuf) -> Result<(), Box<dyn Error>> {
        // Get spec file
        let search: Vec<String> = SearchBuilder::default()
            .location(project)
            .ext("spec")
            .strict()
            .ignore_case()
            .build()
            .collect();
        if search.is_empty() {
            Err("spec file not found")?
        }
        let path_spec = search.iter().next().unwrap();
        let path_spec_docker = path_spec.replace(&project.to_string_lossy().to_string(), "/project");
        // Get arch
        let path_bin_dir = project.join("bin");
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
                            let path_docker = path
                                .to_string_lossy()
                                .replace(&project.to_string_lossy().to_string(), "/project");
                            let command = format!(r#"file {path_docker}"#);
                            match exec::exec_wait("docker", ["exec", &self.container, "/bin/bash", "-c", &command]) {
                                Ok(result) => {
                                    let output = String::from_utf8(result.stdout).unwrap();
                                    if output.contains("ELF") {
                                        if output.contains("x86-64") {
                                            arch = Some(Arch::X86_64.name());
                                        }
                                        if output.contains("aarch64") {
                                            arch = Some(Arch::Aarch64.name());
                                        }
                                        if output.contains("arm") && output.contains("32-bit") {
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
            Err(format!("arch binary file not found"))?
        }
        let arch = arch.unwrap();
        // Exec build
        let command = format!(
            r#"cd /project && rpmbuild -ba --build-in-place --target '{arch}' --define '_topdir /project' {path_spec_docker} && chmod -R 777 /project"#
        );
        print_state!("start build package {}", arch);
        exec::exec_wait("docker", ["exec", &self.container, "/bin/bash", "-c", &command])?;
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
            let path = PathBuf::from(project).join(file);
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

    pub fn rpm_sign(&self, rpm_path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let rpm_name = rpm_path.file_name().unwrap().to_string_lossy();
        let project = &rpm_path.parent().unwrap().to_path_buf();
        let keys = path::get_path_home().join("AuroraOS/package-signing");
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
        // Copy to project
        fs::copy(cert, project.join("regular_cert.pem"))?;
        fs::copy(key, project.join("regular_key.pem"))?;

        print_state!("start sign package '{}'", rpm_name);
        let command = format!(
            r#"rpmsign-external sign --force --key=/project/regular_key.pem --cert=/project/regular_cert.pem /project/{rpm_name}"#
        );
        fn clear(project: &PathBuf) {
            let _ = fs::remove_file(project.join("regular_cert.pem"));
            let _ = fs::remove_file(project.join("regular_key.pem"));
        }
        match exec::exec_wait("docker", ["exec", &self.container, "/bin/bash", "-c", &command]) {
            Ok(_) => clear(project),
            Err(_) => {
                clear(project);
                Err("failed to sign the package")?
            }
        }
        Ok(())
    }
}
