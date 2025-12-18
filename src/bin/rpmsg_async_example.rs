#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    use pru_rproc_user::rpmsg::async_impl::AsyncRpmsg;

    println!("Opening async messaging interface...");
    let rp = match AsyncRpmsg::open_first().await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open async messaging device: {}", e);
            return;
        }
    };

    println!("Async interface opened.");

    loop {
        match rp.read_message_timeout(Some(tokio::time::Duration::from_secs(2))).await {
            Ok(Some(msg)) => {
                if let Ok(s) = std::str::from_utf8(&msg) {
                    println!("PRU -> Host (async): {}", s);
                } else {
                    println!("PRU -> Host (async, hex): {:02X?}", msg);
                }

                if let Err(e) = rp.send(b"ACK").await {
                    eprintln!("Failed to send ACK (async): {}", e);
                }
            }
            Ok(None) => {
                println!("timeout waiting for message (async)");
            }
            Err(e) => {
                eprintln!("async read error: {}", e);
                break;
            }
        }
    }
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("Build with --features async to run the async example");
}
