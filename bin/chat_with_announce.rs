use achat::chat_with_announce::{self, announce_uptime};
use anyhow::Context;
use std::time::Duration;
use tokio::{
    net::TcpListener,
    sync::{broadcast, watch},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .context("Failed to bind on localhost")?;

    let (tx, _rx) = broadcast::channel(16);
    let (topic_tx, topic_rx) = watch::channel("Chat topic".to_string());

    tokio::spawn(announce_uptime(topic_tx, Duration::from_secs(10)));

    loop {
        let (mut socket, addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        let tx = tx.clone();
        let rx = tx.subscribe();
        let topic_rx = topic_rx.clone();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            chat_with_announce::handle_connection(addr, reader, writer, tx, rx, topic_rx)
                .await
                .expect("Failed to handle connection");
        });
    }
}
