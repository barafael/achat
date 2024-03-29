#![warn(missing_docs)]

//! A collection of simple modules which showcase simple use of tasks, channels, and other tokio primitives to
//! implement simple networking applications.

pub use arguments::Arguments;
use std::net::SocketAddr;
use std::time::Duration;

mod arguments;

/// Initialize the console subscriber at the address indicated.
pub fn init_console_subscriber(addr: impl Into<SocketAddr>) {
    console_subscriber::ConsoleLayer::builder()
        .retention(Duration::from_secs(60))
        .server_addr(addr.into())
        .init();
}

/// Is it a quit message?
/// If the `line` is `"quit"` or `"quit\n"` or `"quit\r\n"`, return `true`.
pub fn is_quit(line: &str) -> bool {
    line == "quit" || line == "quit\n" || line == "quit\r\n"
}

/// Broadcast messages sent from one client to all other clients using a [`tokio::sync::broadcast`] channel.
pub mod chat;

/// Broadcast messages sent from one client to all other clients using a [`tokio::sync::broadcast`] channel.
/// Additionally, periodically announce the uptime via a [`tokio::sync::watch`] channel.
pub mod chat_with_announce;

/// Broadcast messages sent from one client to all other clients using a [`tokio::sync::broadcast`] channel.
/// Additionally, commonly share a [`tokio_util::sync::CancellationToken`].
/// Cancelling this token shuts down all the clients (flushing their readers) and then the entire application.
pub mod chat_with_cancel;

/// Collect messages sent from each connected client (via a [`tokio::sync::mpsc`] channel) and store them in a hashmap.
/// On a report request by a client via a [`tokio::sync::oneshot`] channel, send
/// the serialized hashmap.
pub mod collector;

/// Forward messages sent on reader to writer.
pub mod echo;
