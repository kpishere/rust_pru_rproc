use std::process;

use pru_rproc_user::Rpmsg;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/dev/remoteproc/pruss-core0/uevent".to_string());

    println!("Opening uevent path: {}", path);

    let mut rp = match Rpmsg::open_core_by_name(&path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open uevent path: {}", e);
            process::exit(1);
        }
    };

    println!("Opened: {} (uevent={})", rp.path().display(), rp.is_uevent());

    match rp.read_message() {
        Ok(msg) => {
            if msg.is_empty() {
                println!("no message read");
                return;
            }
            if let Ok(s) = std::str::from_utf8(&msg) {
                println!("PRU -> Host: {}", s);
            } else {
                println!("PRU -> Host (hex): {:02X?}", msg);
            }
        }
        Err(e) => eprintln!("read error: {}", e),
    }
}
