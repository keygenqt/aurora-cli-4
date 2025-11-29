use russh::client::Handle;
use russh::client::{self};
use russh::keys::ssh_key;
use russh::keys::PrivateKeyWithHashAlg;
use russh::{ChannelMsg, Preferred};
use russh_sftp::client::SftpSession;
use russh_sftp::protocol::OpenFlags;
use std::borrow::Cow;
use std::error::Error;
use std::fs::File;
use std::fs::{self};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub struct SshClient {}

impl client::Handler for SshClient {
    type Error = russh::Error;

    async fn check_server_key(&mut self, _server_public_key: &ssh_key::PublicKey) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl SshClient {
    pub fn exec(
        command: &str,
        key_path: &PathBuf,
        host: &str,
        user: &str,
        port: u16,
        is_log: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut session = Self::connect(user, host, port, key_path)?;
        Self::call(&mut session, command, is_log)
    }

    pub fn upload(
        path: &PathBuf,
        key_path: &PathBuf,
        host: &str,
        user: &str,
        port: u16,
    ) -> Result<PathBuf, Box<dyn Error>> {
        // Get path upload file
        let to = match path.file_name() {
            Some(name) => PathBuf::from(format!("/home/{}/Downloads/{}", user, name.to_string_lossy())),
            None => Err("error load file name")?,
        };
        let mut session = Self::connect(user, host, port, key_path)?;
        let sftp_session = Self::sftp_session(&mut session)?;
        Self::upload_file(&sftp_session, path, &to)
    }

    fn call(session: &mut Handle<SshClient>, command: &str, is_log: bool,) -> Result<Vec<String>, Box<dyn Error>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::_call(session, command, is_log))
        })
    }

    async fn _call(
        session: &mut Handle<SshClient>,
        command: &str,
        is_log: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut code = None;
        let mut response: Vec<String> = vec![];
        let mut channel = session.channel_open_session().await?;
        channel.exec(true, command).await?;
        loop {
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                ChannelMsg::Data { ref data } => {
                    match str::from_utf8(data.as_ref()) {
                        Ok(out_line) => {
                            let line = out_line.trim().to_string();
                            if is_log {
                                println!("{}", line);
                            }
                            response.push(line)
                        },
                        Err(_) => Err("failed to process SSH connection data")?,
                    };
                }
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                }
                _ => {}
            }
        }
        if let Some(code) = code {
            if code == 1 {
                Err("failed to execute command")?
            }
        }
        Ok(response)
    }

    fn upload_file(session: &SftpSession, from: &PathBuf, to: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::_upload_file(session, from, to))
        })
    }

    async fn _upload_file(session: &SftpSession, from: &PathBuf, to: &PathBuf) -> Result<PathBuf, Box<dyn Error>> {
        let file = File::open(from)?;
        let size = file.metadata()?.len();
        if size == 0 {
            Err("error get size file")?;
        }
        let mut sftp_file = session
            .open_with_flags(
                to.to_string_lossy().to_string(),
                OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE | OpenFlags::READ,
            )
            .await?;
        let data = fs::read(from)?;
        sftp_file.write_all(&data).await?;
        Ok(to.clone())
    }

    fn sftp_session(session: &mut Handle<SshClient>) -> Result<SftpSession, Box<dyn Error>> {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(Self::_sftp_session(session)))
    }

    async fn _sftp_session(session: &mut Handle<SshClient>) -> Result<SftpSession, Box<dyn Error>> {
        let channel = session.channel_open_session().await?;
        channel.request_subsystem(true, "sftp").await.unwrap();
        Ok(SftpSession::new(channel.into_stream()).await?)
    }

    fn connect(user: &str, host: &str, port: u16, key_path: &PathBuf) -> Result<Handle<SshClient>, Box<dyn Error>> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::_connect(user, host, port, key_path))
        })
    }

    async fn _connect(
        user: &str,
        host: &str,
        port: u16,
        key_path: &PathBuf,
    ) -> Result<Handle<SshClient>, Box<dyn Error>> {
        let timeout_session = Duration::from_secs(30);
        let timeout_connect = Duration::from_secs(5);
        let config = client::Config {
            inactivity_timeout: Some(timeout_session),
            preferred: Preferred {
                kex: Cow::Owned(vec![
                    russh::kex::CURVE25519_PRE_RFC_8731,
                    russh::kex::EXTENSION_SUPPORT_AS_CLIENT,
                ]),
                ..Default::default()
            },
            ..<_>::default()
        };
        let config = Arc::new(config);
        let sh = SshClient {};
        let mut session = match tokio::time::timeout(timeout_connect, client::connect(config, (host, port), sh)).await?
        {
            Ok(session) => session,
            Err(err) => Err(err)?,
        };
        let secret_key = Arc::new(russh::keys::load_secret_key(key_path, None)?);
        let key_pair = PrivateKeyWithHashAlg::new(secret_key, session.best_supported_rsa_hash().await?.flatten());
        let result = session.authenticate_publickey(user, key_pair).await?;
        if !result.success() {
            Err("failed to connect to the emulator via SSH")?
        }
        Ok(session)
    }
}
