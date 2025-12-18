use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use libc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpmsgError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("no rpmsg devices found")]
    NotFound,
}

#[derive(Debug)]
pub struct Rpmsg {
    path: PathBuf,
    file: File,
    /// true when the underlying file is an /dev/remoteproc uevent file (read-only lines)
    is_uevent: bool,
}

impl Rpmsg {
    /// List candidate rpmsg device names from /dev (e.g. "rpmsg0", "rpmsg_pru0").
    pub fn list() -> Result<Vec<String>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(Path::new("/dev"))? {
            let e = entry?;
            if let Some(name) = e.file_name().to_str() {
                if name.starts_with("rpmsg") {
                    out.push(name.to_string());
                }
            }
        }
        Ok(out)
    }

    /// Open an rpmsg device by its /dev name (e.g. "rpmsg_pru0").
    pub fn open(name: &str) -> Result<Self> {
        let path = Path::new("/dev").join(name);
        let file = File::options().read(true).write(true).open(&path)?;
        Ok(Rpmsg { path, file, is_uevent: false })
    }

    /// Paths to remoteproc-style uevent devices which drivers may expose.
    /// Try both `pruss-core0` and `pruss-core1`.
    const UEVENT_PATHS: [&'static str; 2] = [
        "/dev/remoteproc/pruss-core0/uevent",
        "/dev/remoteproc/pruss-core1/uevent",
    ];

    /// Open the first available messaging interface. Prefer the remoteproc uevent
    /// path if present, otherwise fall back to `/dev/rpmsg*` devices.
    pub fn open_first() -> Result<Self> {
        for p in Self::UEVENT_PATHS.iter() {
            let uevent = Path::new(p);
            if uevent.exists() {
                let file = File::options().read(true).open(uevent)?;
                return Ok(Rpmsg { path: uevent.to_path_buf(), file, is_uevent: true });
            }
        }

        let list = Self::list()?;
        let name = list.get(0).ok_or(RpmsgError::NotFound)?;
        Self::open(name)
    }

    /// Open a specific remoteproc core's uevent interface if present.
    ///
    /// `core` selects an index into the built-in `UEVENT_PATHS` array
    /// (0 => `pruss-core0`, 1 => `pruss-core1`). Returns `NotFound` when the
    /// requested core path does not exist.
    pub fn open_core(core: usize) -> Result<Self> {
        match Self::UEVENT_PATHS.get(core) {
            Some(p) => {
                let uevent = Path::new(p);
                if uevent.exists() {
                    let file = File::options().read(true).open(uevent)?;
                    return Ok(Rpmsg { path: uevent.to_path_buf(), file, is_uevent: true });
                }
                Err(RpmsgError::NotFound)
            }
            None => Err(RpmsgError::NotFound),
        }
    }

    /// Open a uevent path by arbitrary filesystem path (e.g. "/dev/remoteproc/pruss-core1/uevent").
    pub fn open_core_by_name<P: AsRef<Path>>(path: P) -> Result<Self> {
        let p = path.as_ref();
        if p.exists() {
            let file = File::options().read(true).open(p)?;
            return Ok(Rpmsg { path: p.to_path_buf(), file, is_uevent: true });
        }
        Err(RpmsgError::NotFound)
    }

    /// Blocking read into the provided buffer. Returns number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.file.read(buf)?;
        Ok(n)
    }

    /// Convenience: read a single message into a Vec. For uevent files this reads a
    /// single line (blocking until a newline); for rpmsg devices it reads raw bytes.
    pub fn read_message(&mut self) -> Result<Vec<u8>> {
        if self.is_uevent {
            let mut reader = BufReader::new(&self.file);
            let mut line = String::new();
            let n = reader.read_line(&mut line)?;
            let mut v = line.into_bytes();
            v.truncate(n);
            Ok(v)
        } else {
            let mut buf = vec![0u8; 4096];
            let n = self.read(&mut buf)?;
            buf.truncate(n);
            Ok(buf)
        }
    }

    /// Read a message with a timeout. Returns `Ok(Some(msg))` when a message
    /// was read, `Ok(None)` on timeout, or `Err` on error. Use `None` for
    /// blocking behavior (same as `read_message`).
    pub fn read_message_timeout(&mut self, timeout: Option<Duration>) -> Result<Option<Vec<u8>>> {
        let fd = self.file.as_raw_fd();
        let mut pfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };

        let timeout_ms: i32 = match timeout {
            Some(dur) => {
                let ms = dur.as_millis();
                if ms > i32::MAX as u128 { i32::MAX } else { ms as i32 }
            }
            None => -1,
        };

        let res = unsafe { libc::poll(&mut pfd as *mut libc::pollfd, 1, timeout_ms) };
        if res < 0 {
            return Err(RpmsgError::Io(io::Error::last_os_error()));
        }
        if res == 0 {
            return Ok(None);
        }

        // readable; delegate to read_message which handles uevent vs rpmsg reads
        let msg = self.read_message()?;
        Ok(Some(msg))
    }

    /// Send bytes to the PRU over rpmsg.
    pub fn send(&mut self, data: &[u8]) -> Result<usize> {
        if self.is_uevent {
            return Err(RpmsgError::Io(io::Error::new(
                io::ErrorKind::Other,
                "send not supported on uevent",
            )));
        }
        let n = self.file.write(data)?;
        Ok(n)
    }

    /// Return the device path opened.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return true if the opened interface is the remoteproc `uevent` file.
    pub fn is_uevent(&self) -> bool {
        self.is_uevent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_ok() {
        // This test only ensures the call runs on systems without /dev entries.
        let _ = Rpmsg::list();
    }

    #[test]
    fn open_core_not_found() {
        // Attempt to open a core index that's unlikely to exist in CI; should return NotFound.
        match Rpmsg::open_core(99) {
            Err(RpmsgError::NotFound) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }

    #[test]
    fn open_core_by_name_not_found() {
        match Rpmsg::open_core_by_name("/this/path/does/not/exist") {
            Err(RpmsgError::NotFound) => {}
            other => panic!("expected NotFound, got {:?}", other),
        }
    }
}

pub type Result<T> = std::result::Result<T, RpmsgError>;

#[cfg(feature = "async")]
pub mod async_impl {
    use super::*;
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;
    use tokio::io::unix::{AsyncFd, AsyncFdReadyGuard};

    /// Async wrapper around nonblocking file descriptors for rpmsg/uevent.
    /// Uses `tokio::io::unix::AsyncFd` for readiness notifications and performs
    /// non-blocking `read`/`write` syscalls on the underlying `File`.
    pub struct AsyncRpmsg {
        fd: AsyncFd<File>,
        path: PathBuf,
        is_uevent: bool,
    }

    impl AsyncRpmsg {
        /// Open the first available messaging interface with O_NONBLOCK.
        pub async fn open_first() -> Result<Self> {
            // prefer uevent if present
            for p in Rpmsg::UEVENT_PATHS.iter() {
                let uevent = Path::new(p);
                if uevent.exists() {
                    let file = OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_NONBLOCK)
                        .open(uevent)?;
                    let afd = AsyncFd::new(file)?;
                    return Ok(AsyncRpmsg { fd: afd, path: uevent.to_path_buf(), is_uevent: true });
                }
            }

            // fallback to first /dev/rpmsg* device (read/write)
            let list = Rpmsg::list()?;
            let name = list.get(0).ok_or(RpmsgError::NotFound)?;
            let path = Path::new("/dev").join(name);
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&path)?;
            let afd = AsyncFd::new(file)?;
            Ok(AsyncRpmsg { fd: afd, path, is_uevent: false })
        }

        /// Read a single message (blocking until available) asynchronously.
        pub async fn read_message(&self) -> Result<Vec<u8>> {
            // Delegate to read_message_timeout with None (block forever).
            match self.read_message_timeout(None).await? {
                Some(v) => Ok(v),
                None => Ok(Vec::new()),
            }
        }

        /// Read a message with optional timeout. `timeout` is `None` for blocking.
        pub async fn read_message_timeout(&self, timeout: Option<tokio::time::Duration>) -> Result<Option<Vec<u8>>> {
            // Await readiness with optional timeout
            let ready_fut = self.fd.readable();
            let mut guard: AsyncFdReadyGuard<'_, File> = if let Some(dur) = timeout {
                match tokio::time::timeout(dur, ready_fut).await {
                    Ok(Ok(g)) => g,
                    Ok(Err(e)) => return Err(RpmsgError::Io(io::Error::new(io::ErrorKind::Other, e.to_string()))),
                    Err(_) => return Ok(None),
                }
            } else {
                ready_fut.await.map_err(|e| RpmsgError::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))?
            };

            // Try a non-blocking read. If it would block, clear readiness and wait again.
            let mut buf = vec![0u8; 4096];
            let res = guard.try_io(|inner: &AsyncFd<File>| {
                // perform a non-blocking read syscall directly on the fd
                let fd = inner.get_ref().as_raw_fd();
                let r = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                if r < 0 {
                    let err = io::Error::last_os_error();
                    return Err(err);
                }
                Ok(r as usize)
            });

            match res {
                Ok(Ok(n)) => {
                    if n == 0 {
                        return Ok(Some(Vec::new()));
                    }
                    buf.truncate(n);
                    return Ok(Some(buf));
                }
                Ok(Err(e)) => return Err(RpmsgError::Io(e)),
                Err(_would_block) => {
                    // try_io indicated the fd would block; return None to indicate no data
                    return Ok(None);
                }
            }
        }

        /// Send data to PRU. Not supported for uevent-backed interfaces.
        pub async fn send(&self, data: &[u8]) -> Result<usize> {
            if self.is_uevent {
                return Err(RpmsgError::Io(io::Error::new(io::ErrorKind::Other, "send not supported on uevent")));
            }

            // Wait for writable readiness
            let mut guard: AsyncFdReadyGuard<'_, File> = self.fd.writable().await.map_err(|e| RpmsgError::Io(io::Error::new(io::ErrorKind::Other, e.to_string())))?;
            let res = guard.try_io(|inner: &AsyncFd<File>| {
                let fd = inner.get_ref().as_raw_fd();
                let w = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, data.len()) };
                if w < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(w as usize)
            });

            match res {
                Ok(Ok(n)) => Ok(n),
                Ok(Err(e)) => Err(RpmsgError::Io(e)),
                Err(_would_block) => Ok(0),
            }
        }

        pub fn path(&self) -> &Path {
            &self.path
        }

        pub fn is_uevent(&self) -> bool {
            self.is_uevent
        }
        
        /// Open a specific core index asynchronously (0 => core0, 1 => core1).
        pub async fn open_core(core: usize) -> Result<Self> {
            if let Some(p) = Rpmsg::UEVENT_PATHS.get(core) {
                let uevent = Path::new(p);
                if uevent.exists() {
                    let file = OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_NONBLOCK)
                        .open(uevent)?;
                    let afd = AsyncFd::new(file)?;
                    return Ok(AsyncRpmsg { fd: afd, path: uevent.to_path_buf(), is_uevent: true });
                }
                return Err(RpmsgError::NotFound);
            }
            Err(RpmsgError::NotFound)
        }

        /// Open a uevent path by arbitrary filesystem path asynchronously.
        pub async fn open_core_by_name(path: &str) -> Result<Self> {
            let p = Path::new(path);
            if p.exists() {
                let file = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(p)?;
                let afd = AsyncFd::new(file)?;
                return Ok(AsyncRpmsg { fd: afd, path: p.to_path_buf(), is_uevent: true });
            }
            Err(RpmsgError::NotFound)
        }
    }
}
