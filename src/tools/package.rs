use std::path::PathBuf;

pub struct Package {}

impl Package {
    pub fn get_package_name(path: &PathBuf) -> Option<String> {
        let package = match rpm::Package::open(path) {
            Ok(value) => value,
            Err(_) => return None,
        };
        match package.metadata.get_name() {
            Ok(value) => Some(value.to_string()),
            Err(_) => None,
        }
    }
}