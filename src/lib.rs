pub mod remoteproc;
pub mod mmio;
pub mod rpmsg;

pub use remoteproc::{RemoteProc, RemoteProcError, RemoteProcState};
pub use mmio::{Mmio, MmioError};
pub use rpmsg::{Rpmsg, RpmsgError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        // basic compilation check
        let _ = RemoteProc::list();
    }
}
