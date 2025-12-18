use std::process;
use std::str;

use pru_rproc_user::Rpmsg;

fn main() {
    println!("Opening messaging interface (prefers remoteproc uevent)...");

    let mut rp = match Rpmsg::open_first() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open messaging device: {}", e);
            process::exit(1);
        }
    };

    println!("Opened device: {} (uevent={})", rp.path().display(), rp.is_uevent());

    loop {
        match rp.read_message() {
            Ok(msg) => {
                if msg.is_empty() {
                    continue;
                }
                // try to print as UTF-8, else hex
                match str::from_utf8(&msg) {
                    Ok(s) => println!("PRU -> Host: {}", s),
                    Err(_) => println!("PRU -> Host (hex): {:02X?}", msg),
                }

                // send an acknowledgement back to the PRU
                if let Err(e) = rp.send(b"ACK") {
                    eprintln!("Failed to send ACK: {}", e);
                }
            }
            Err(e) => {
                eprintln!("rpmsg read error: {}", e);
                break;
            }
        }
    }
}
