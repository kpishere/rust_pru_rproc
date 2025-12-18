#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    use pru_rproc_user::rpmsg::async_impl::AsyncRpmsg;

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/dev/remoteproc/pruss-core0/uevent".to_string());

    println!("Opening async uevent path: {}", path);

    let rp = match AsyncRpmsg::open_core_by_name(&path).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open async uevent path: {}", e);
            std::process::exit(1);
        }
    };

    println!("Opened async: {} (uevent={})", rp.path().display(), rp.is_uevent());

    match rp.read_message_timeout(Some(tokio::time::Duration::from_secs(2))).await {
        Ok(Some(msg)) => {
            if let Ok(s) = std::str::from_utf8(&msg) {
                println!("PRU -> Host (async): {}", s);
            } else {
                println!("PRU -> Host (async, hex): {:02X?}", msg);
            }
        }
        Ok(None) => println!("timeout waiting for message (async)"),
        Err(e) => eprintln!("async read error: {}", e),
    }
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("Build with --features async to run the async example");
}
