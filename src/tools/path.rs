use std::env;
use std::path::PathBuf;

/// Get home folder
pub fn get_path_home() -> PathBuf {
    env::home_dir().unwrap_or_else(|| panic!("Failed to get HOME path"))
}

/// Get current folder
pub fn get_path_current() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| panic!("Failed to get PWD path"))
}

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
