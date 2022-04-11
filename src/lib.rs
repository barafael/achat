#![warn(missing_docs)]

//! A collection of simple modules which showcase simple use of tasks, channels, and other tokio primitives to
//! implement simple networking applications.

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
