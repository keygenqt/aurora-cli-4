use std::ffi::OsStr;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;

pub fn exec_wait<I, S>(program: &str, args: I) -> Result<Output, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => Ok(output),
        Err(_) => Err("exec team failed")?,
    }
}
