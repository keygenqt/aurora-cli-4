use std::error::Error;
use std::path::PathBuf;

use crate::exit_error;
use crate::tools::emulator::Emulator;
use crate::tools::package::Package;
use crate::tools::path;

pub struct EmulatorPackageInstall {}

impl EmulatorPackageInstall {
    pub fn exec(path: PathBuf) -> Result<String, Box<dyn Error>> {
        let path = match path::get_relative_path_canonicalize(&path) {
            Some(path) => path,
            None => exit_error!("package not found"),
        };
        let package = match Package::get_package_name(&path) {
            Some(package) => package,
            None => exit_error!("failed to get package name"),
        };
        let emulator = Emulator::default();
        let upload = emulator.upload(&path)?;
        emulator.uninstall(&package)?;
        emulator.install(&upload)?;
        Ok(package)
    }
}
