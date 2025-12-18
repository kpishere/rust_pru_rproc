use std::process;

use pru_rproc_user::Mmio;
use pru_rproc_user::mmio::PRU0_DRAM_BASE;

fn main() {
    println!("Mapping PRU0 DRAM (requires root)...");

    let mut mm = match Mmio::map_pru0_dram() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to map PRU0 DRAM: {}", e);
            process::exit(1);
        }
    };

    let addr = PRU0_DRAM_BASE;
    let test_val: u32 = 0xDEADBEEF;

    println!("Writing 0x{:08X} to {:#X}", test_val, addr);
    mm.write_u32(addr, test_val);

    let read_back = mm.read_u32(addr);
    println!("Read back 0x{:08X}", read_back);

    if read_back == test_val {
        println!("Round-trip OK");
    } else {
        println!("Mismatch: wrote 0x{:08X}, read 0x{:08X}", test_val, read_back);
    }
}
