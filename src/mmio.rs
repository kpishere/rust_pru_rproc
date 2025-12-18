use memmap2::{MmapMut, MmapOptions};
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MmioError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("map error: {0}")]
    Map(String),
}

pub struct Mmio {
    map: MmapMut,
    base: u64,
}

// Common AM335x / BeagleBone Black PRU/PRUSS addresses.
// These are provided as convenience defaults; verify against your board/device tree.
pub const PRUSS_BASE: u64 = 0x4A300000;
pub const PRUSS_SIZE: usize = 0x0002_0000; // 128 KiB (typical PRUSS region size)

pub const PRU0_DRAM_BASE: u64 = 0x4A310000;
pub const PRU0_DRAM_SIZE: usize = 0x0000_2000; // 8 KiB

pub const PRU1_DRAM_BASE: u64 = 0x4A320000;
pub const PRU1_DRAM_SIZE: usize = 0x0000_2000; // 8 KiB

impl Mmio {
    /// Map `len` bytes starting at physical `base`. Requires root privileges.
    pub fn map(base: u64, len: usize) -> Result<Self> {
        let dev = OpenOptions::new().read(true).write(true).open(Path::new("/dev/mem"))?;
        let page_offset = (base as usize) & (4096 - 1);
        let aligned_base = base - (page_offset as u64);
        let aligned_len = len + page_offset;

        let map = unsafe {
            MmapOptions::new()
                .offset(aligned_base)
                .len(aligned_len)
                .map_mut(&dev)
                .map_err(|e| MmioError::Map(e.to_string()))?
        };

        Ok(Mmio { map, base })
    }

    /// Convenience: map the whole PRUSS (ICSS) region using the common BBB address.
    pub fn map_pruss() -> Result<Self> {
        Mmio::map(PRUSS_BASE, PRUSS_SIZE)
    }

    /// Convenience: map PRU0 data RAM (verify address for your board).
    pub fn map_pru0_dram() -> Result<Self> {
        Mmio::map(PRU0_DRAM_BASE, PRU0_DRAM_SIZE)
    }

    /// Convenience: map PRU1 data RAM (verify address for your board).
    pub fn map_pru1_dram() -> Result<Self> {
        Mmio::map(PRU1_DRAM_BASE, PRU1_DRAM_SIZE)
    }

    fn offset(&self, addr: u64) -> usize {
        (addr - self.base) as usize
    }

    pub fn read_u32(&self, addr: u64) -> u32 {
        let off = self.offset(addr);
        let bytes = &self.map[off..off + 4];
        u32::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn write_u32(&mut self, addr: u64, val: u32) {
        let off = self.offset(addr);
        let bytes = val.to_le_bytes();
        self.map[off..off + 4].copy_from_slice(&bytes);
    }
}

pub type Result<T> = std::result::Result<T, MmioError>;
