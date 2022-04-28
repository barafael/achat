#![warn(missing_docs)]

//! A collection of simple modules which showcase simple use of tasks, channels, and other tokio primitives to
//! implement simple networking applications.

use anyhow::Context;
use clap::Parser;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

/// Command Line Arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Address to listen on.
    #[clap(short, long)]
    pub address: String,

    /// Address to publish console events on.
    #[clap(short, long)]
    pub console: Option<String>,
}

/// Initialize the console subscriber at the address indicated.
pub fn init_console_subscriber(addr: String) {
    let console: SocketAddr = addr.parse().unwrap();
    console_subscriber::ConsoleLayer::builder()
        .retention(Duration::from_secs(60))
        .server_addr(console)
        .init();
}

/// Ask for name on `socket`.
pub async fn get_name<S>(socket: &mut S) -> anyhow::Result<String>
where
    S: AsyncWrite + AsyncRead + Unpin,
{
    let mut socket = BufReader::new(socket);

    socket
        .write_all(b"Enter your name: ")
        .await
        .context("Failed to ask for name")?;

    let mut name = String::new();
    socket
        .read_line(&mut name)
        .await
        .context("Failed to receive name")?;
    if name.ends_with('\n') {
        name.pop().unwrap();
    }
    if name.ends_with('\r') {
        name.pop().unwrap();
    }
    Ok(name)
}

/// Broadcast messages sent from one client to all other clients using a [`tokio::sync::broadcast`] channel.
pub mod chat;

/// Broadcast messages sent from one client to all other clients using a [`tokio::sync::broadcast`] channel.
/// Additionally, periodically announce the uptime via a [`tokio::sync::watch`] channel.
pub mod chat_with_announce;

/// Collect messages sent from each connected client (via a [`tokio::sync::mpsc`] channel) and store them in a hashmap.
/// On a report request by a client via a [`tokio::sync::oneshot`] channel, send
/// the serialized hashmap.
pub mod collector;

/// Forward messages sent on reader to writer.
pub mod echo;
