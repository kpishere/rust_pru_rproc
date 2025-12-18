use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

const SYS_REMOTEPROC: &str = "/sys/class/remoteproc";

#[derive(Debug, Error)]
pub enum RemoteProcError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("invalid state data")]
    InvalidState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteProcState {
    Offline,
    Booting,
    Online,
    Suspended,
    Unknown(String),
}

impl RemoteProcState {
    fn from_str(s: &str) -> Self {
        match s.trim() {
            "offline" => RemoteProcState::Offline,
            "booting" => RemoteProcState::Booting,
            "online" => RemoteProcState::Online,
            "suspended" => RemoteProcState::Suspended,
            other => RemoteProcState::Unknown(other.to_string()),
        }
    }
}

pub struct RemoteProc {
    path: PathBuf,
}

impl RemoteProc {
    pub fn list() -> Result<Vec<String>> {
        let mut out = Vec::new();
        let base = Path::new(SYS_REMOTEPROC);
        if base.exists() {
            for entry in fs::read_dir(base)? {
                let e = entry?;
                if let Some(name) = e.file_name().to_str() {
                    out.push(name.to_string());
                }
            }
        }
        Ok(out)
    }

    pub fn open(name: &str) -> Result<Self> {
        let path = Path::new(SYS_REMOTEPROC).join(name);
        if !path.exists() {
            return Err(RemoteProcError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                format!("remoteproc '{}' not found", name),
            )));
        }
        Ok(RemoteProc { path })
    }

    fn read_attr(&self, attr: &str) -> Result<String> {
        let p = self.path.join(attr);
        let s = fs::read_to_string(p)?;
        Ok(s)
    }

    fn write_attr(&self, attr: &str, data: &str) -> Result<()> {
        let p = self.path.join(attr);
        let mut f = fs::OpenOptions::new().write(true).open(p)?;
        f.write_all(data.as_bytes())?;
        Ok(())
    }

    pub fn state(&self) -> Result<RemoteProcState> {
        let s = self.read_attr("state")?;
        Ok(RemoteProcState::from_str(&s))
    }

    /// Set the `firmware` file for the remoteproc. This writes the string
    /// firmware filename which the kernel will use when starting the remoteproc.
    pub fn set_firmware<P: AsRef<Path>>(&self, firmware: P) -> Result<()> {
        let s = firmware.as_ref().to_string_lossy();
        self.write_attr("firmware", &s)
    }

    /// Start the remoteproc by writing `start` to `state`.
    pub fn start(&self) -> Result<()> {
        self.write_attr("state", "start")
    }

    /// Stop the remoteproc by writing `stop` to `state`.
    pub fn stop(&self) -> Result<()> {
        self.write_attr("state", "stop")
    }

    /// Remove/unbind the remoteproc device (write 'remove').
    pub fn remove(&self) -> Result<()> {
        self.write_attr("state", "remove")
    }
}

pub type Result<T> = std::result::Result<T, RemoteProcError>;
