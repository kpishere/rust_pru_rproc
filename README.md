pru_rproc_user
=================

 - This crate performs sysfs writes and /dev/mem mapping — these require root privileges.
 - The library intentionally does not hardcode PRU physical addresses; map the regions you need manually (addresses vary by board/SoC).
 - For receiving notifications (interrupts) or messages from the PRU, the crate exposes a simple `rpmsg` helper that opens `/dev/rpmsg*` devices and can read/write messages.
 - See https://glennklockwood.com/embedded/beaglebone-pru.html and https://github.com/sbarral/prusst for conceptual references.
 - For receiving notifications (interrupts) or messages from the PRU, the crate exposes a simple `rpmsg` helper that opens `/dev/rpmsg*` devices and can read/write messages.
 - Convenience MMIO mapping helpers are provided in `Mmio` for common BeagleBone Black PRU regions (`map_pruss`, `map_pru0_dram`, `map_pru1_dram`). Verify these addresses against your device tree before use.
 - See https://glennklockwood.com/embedded/beaglebone-pru.html and https://github.com/sbarral/prusst for conceptual references.
 - For receiving notifications (interrupts) or messages from the PRU, the crate exposes a simple `rpmsg` helper that opens `/dev/rpmsg*` devices and can read/write messages.
 - The crate prefers the remoteproc uevent interface when present. It will look for either `/dev/remoteproc/pruss-core0/uevent` or `/dev/remoteproc/pruss-core1/uevent` and use the first one found.
 - You can open a specific core explicitly with `Rpmsg::open_core(core_index)` (0 => core0, 1 => core1) or use `Rpmsg::open_first()` to auto-select.
 - Async support: enable the `async` Cargo feature to use `rpmsg::async_impl::AsyncRpmsg` which exposes `read_message`, `read_message_timeout` and `send` as async methods. The async implementation uses non-blocking FDs and `tokio::io::unix::AsyncFd`.
 - You can open a specific core explicitly with `Rpmsg::open_core(core_index)` (0 => core0, 1 => core1), use `Rpmsg::open_first()` to auto-select, or open an arbitrary uevent path with `Rpmsg::open_core_by_name(path)`.
 - Async support: enable the `async` Cargo feature to use `rpmsg::async_impl::AsyncRpmsg` which exposes `read_message`, `read_message_timeout` and `send` as async methods. The async implementation uses non-blocking FDs and `tokio::io::unix::AsyncFd`.

Examples

Open core 1 synchronously:

```rust
use pru_rproc_user::Rpmsg;

let rp = Rpmsg::open_core(1)?; // attempt pruss-core1
println!("opened: {}", rp.path().display());
```

Open arbitrary uevent path synchronously:

```rust
let rp = Rpmsg::open_core_by_name("/dev/remoteproc/pruss-core1/uevent")?;
```

Async example (feature `async`):

```rust
use pru_rproc_user::rpmsg::async_impl::AsyncRpmsg;
let rp = AsyncRpmsg::open_core(1).await?;
let maybe_msg = rp.read_message_timeout(Some(tokio::time::Duration::from_secs(2))).await?;
```

Binaries / Examples

This crate includes small example binaries under `src/bin/` demonstrating common flows.

- `open_core_example`: open an arbitrary uevent path synchronously and read a single message.
- `open_core_async_example`: async variant (build with `--features async`).

Build and run examples (requires root and device with PRU remoteproc/uevent):

```bash
# build binaries (async feature optional)
cargo build --bins --features async

# run sync example (default path is /dev/remoteproc/pruss-core0/uevent)
sudo cargo run --bin open_core_example -- /dev/remoteproc/pruss-core1/uevent

# run async example (requires --features async)
sudo cargo run --bin open_core_async_example --features async -- /dev/remoteproc/pruss-core1/uevent
```
 - Convenience MMIO mapping helpers are provided in `Mmio` for common BeagleBone Black PRU regions (`map_pruss`, `map_pru0_dram`, `map_pru1_dram`). Verify these addresses against your device tree before use.
 - See https://glennklockwood.com/embedded/beaglebone-pru.html and https://github.com/sbarral/prusst for conceptual references.
- See https://glennklockwood.com/embedded/beaglebone-pru.html and https://github.com/sbarral/prusst for conceptual references.
# rust_pru_rproc
A rust implementation of a userland interface to TI PRU devices like on the Beaglebone Black 
