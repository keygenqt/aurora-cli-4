use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug)]
pub struct ExecResult {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

impl ExecResult {
    pub fn contains(&self, value: &str) -> bool {
        for line in self.stdout.iter() {
            if line.contains(value) {
                return true;
            }
        }
        for line in self.stderr.iter() {
            if line.contains(value) {
                return true;
            }
        }
        false
    }
}

pub struct Exec {}

impl Exec {
    pub fn command<I, S>(program: &str, args: I) -> Result<ExecResult, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Self::_command(program, args, false)
    }

    pub fn command_verbose<I, S>(program: &str, args: I) -> Result<ExecResult, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Self::_command(program, args, true)
    }

    fn _command<I, S>(program: &str, args: I, verbose: bool) -> Result<ExecResult, Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut child = Command::new(program)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        let mut resout = Vec::<String>::new();
        let mut reserr = Vec::<String>::new();

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        let reader_out = BufReader::new(stdout);
        let reader_err = BufReader::new(stderr);

        for line in reader_out.lines() {
            if let Ok(line) = line {
                resout.push(line.clone());
                if verbose {
                    println!("{}", line);
                }
            }
        }

        for line in reader_err.lines() {
            if let Ok(line) = line {
                reserr.push(line.clone());
                if verbose {
                    println!("{}", line);
                }
            }
        }

        let status = child.wait().expect("Failed to wait for child process");
        if !status.success() {
            Err(resout.join("\n"))?
        } else {
            Ok(ExecResult {
                stdout: resout.clone(),
                stderr: resout.clone(),
            })
        }
    }
}
