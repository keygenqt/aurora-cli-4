use std::env;
use std::path::Path;
use std::path::PathBuf;

use walkdir::WalkDir;

/// Get home folder
pub fn get_path_home() -> PathBuf {
    env::home_dir().unwrap_or_else(|| panic!("Failed to get HOME path"))
}

/// Get current folder
pub fn get_path_current() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| panic!("Failed to get PWD path"))
}

#[cfg(unix)]
pub fn get_relative_path(path: &PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy().to_string();
    // Check relative path
    let path_str = if !path_str.starts_with("./") && !path_str.starts_with("~/") && !path_str.starts_with("/") {
        format!("{}/", get_path_current().to_string_lossy())
    } else {
        path_str
    };
    // Check relative path
    let path_str = if path_str.starts_with("./") {
        path_str.replacen("./", &format!("{}/", get_path_current().to_string_lossy()), 1)
    } else {
        path_str
    };
    // Check home dir
    let path_str = if path_str.starts_with("~/") {
        path_str.replacen("~/", &format!("{}/", &get_path_home().to_string_lossy()), 1)
    } else {
        path_str
    };
    PathBuf::from(&path_str)
}

#[cfg(windows)]
pub fn get_relative_path(path: &PathBuf) -> PathBuf {
    let path_str = windows_clear(&path.to_string_lossy().to_string());
    // Check relative path
    let path_str = if !path_str.contains(":")
        && !path_str.starts_with(".\\")
        && !path_str.starts_with("~\\")
        && !path_str.starts_with("\\")
        && !path_str.contains(":\\")
    {
        format!("{}\\", get_path_current().to_string_lossy())
    } else {
        path_str
    };
    // Check relative path
    let path_str = if path_str.starts_with(".\\") {
        path_str.replacen(".\\", &format!("{}\\", get_path_current().to_string_lossy()), 1)
    } else {
        path_str
    };
    // Check home dir
    let path_str = if path_str.starts_with("~\\") {
        path_str.replacen("~\\", &format!("{}\\", &get_path_home().to_string_lossy()), 1)
    } else {
        path_str
    };
    PathBuf::from(&path_str)
}

/// Get relative path
pub fn get_relative_path_canonicalize(path: &PathBuf) -> Option<PathBuf> {
    let path = get_relative_path(&path);
    if !path.exists() {
        None
    } else {
        match path.canonicalize() {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

/// Resolve container path to host path for Docker-in-Docker volume mounts.
pub fn get_resolve_host_path<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    if let Ok(mount_env) = env::var("AURORA_CLI_MOUNT") {
        if let Some((container_root, host_root)) = mount_env.split_once(':') {
            let path_str = path.to_string_lossy().to_string();
            if path_str.starts_with(container_root) {
                let relative = path_str.strip_prefix(container_root).unwrap();
                let resolved = PathBuf::from(host_root).join(relative.trim_start_matches('/'));
                return resolved.to_string_lossy().to_string();
            }
        }
    }
    path.to_string_lossy().to_string()
}

pub fn search_files_by_ext(path: &PathBuf, ext: &str) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let path_str = windows_clear(&path.to_string_lossy().to_string());
        if !path_str.contains("/Trash/")
            && !path_str.contains("/target/")
            && !path_str.contains("\\target\\")
            && path.is_file()
        {
            match path.extension() {
                Some(extension) => {
                    if extension.to_str().unwrap() == ext {
                        result.push(path_str);
                    }
                }
                None => {}
            }
        }
    }
    result
}

pub fn windows_clear(path: &str) -> String {
    path.replace("\\\\?\\", "")
}

pub fn windows_convert_to_unix(path: &str) -> String {
    path.replace("\\", "/")
}

#[allow(dead_code)]
pub fn windows_convert_from_unix(path: &str) -> String {
    path.replace("/", "\\").replace("/c", "C:")
}
