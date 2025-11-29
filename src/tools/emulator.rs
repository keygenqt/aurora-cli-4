use std::{error::Error, path::PathBuf};

use crate::tools::{constants, path, ssh::SshClient};

pub struct Emulator {
    pub host: String,
    pub user: String,
    pub port: u16,
    pub key: PathBuf,
}

impl Default for Emulator {
    fn default() -> Self {
        let prepared_path = path::windows_clear(constants::SSH_EMULATOR_KEY_PATH);
        let key_path = path::get_relative_path(&PathBuf::from(prepared_path));
        let key_path = path::get_relative_path_canonicalize(&key_path).expect("failed to obtain connection key");
        Self {
            host: "localhost".into(),
            user: "defaultuser".into(),
            port: 2223,
            key: key_path
        }
    }
}

impl Emulator {
    pub fn run(&self, package: &str) -> Result<(), Box<dyn Error>> {
        let command = format!("runtime-manager-tool Control startDebug {} --output-to-console", package);
        let _ = SshClient::exec(&command, &self.key, &self.host, &self.user, self.port, true);
        Ok(())
    }

    pub fn upload(&self, path: &PathBuf) -> Result<PathBuf, Box<dyn Error>>  {
        SshClient::upload(&path, &self.key, &self.host, &self.user, self.port)
    }

    pub fn uninstall(&self, package: &str) -> Result<(), Box<dyn Error>> {
        let prompt = "\"{'KeepUserData': <true>}\"";
        let command = format!(
            "gdbus call --system --dest ru.omp.APM --object-path /ru/omp/APM --method ru.omp.APM.Remove \"{package}\" {}",
            prompt
        );
        let _ = SshClient::exec(&command, &self.key, &self.host, &self.user, self.port, false);
        Ok(())
    }

    pub fn install(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        let prompt = "\"{}\"";
        let command = format!(
            "gdbus call --system --dest ru.omp.APM --object-path /ru/omp/APM --method ru.omp.APM.Install \"{}\" {}",
            path.to_string_lossy(),
            prompt
        );
        let result = SshClient::exec(&command, &self.key, &self.host, &self.user, self.port, false)?;
        if result.len() > 0 && result.iter().next().unwrap() == "()" {
            Ok(())
        } else {
            Err(result.join("\n"))?
        }
    }
}