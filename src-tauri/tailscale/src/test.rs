use std::error::Error;

use crate::{Tailscale, TailscaleError};

#[test]
fn start_listener() -> Result<(), TailscaleError> {
    println!("Creating server");
    // Create a new server
    let ts = Tailscale::new();

    // Configure it
    println!("Configuring directory");
    ts.set_dir("/tmp/tailscale-rust-test")?;
    println!("Configuring hostname");
    ts.set_hostname("my-rust-node")?;
    println!("Setting ephemeral");
    //ts.set_authkey("tskey-...")?; // Set authkey if needed for auto-registration
    ts.set_ephemeral(true)?;

    // Bring the server up
    println!("Starting Tailscale...");
    ts.up()?;
    println!("Tailscale started!");

    // Get IPs
    let mut ip_buf = [0u8; 256];
    let ips = ts.get_ips(&mut ip_buf)?;
    println!("Tailscale IPs: {}", ips);
    Ok(())
}
