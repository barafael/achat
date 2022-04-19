use achat::chat;
use anyhow::Context;
use std::time::Duration;
use tokio::{net::TcpListener, sync::broadcast};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::ConsoleLayer::builder()
        .retention(Duration::from_secs(60))
        .server_addr(([127, 0, 0, 1], 6669))
        .init();

    let listener = TcpListener::bind("localhost:8080")
        .await
        .context("Failed to bind on localhost")?;

    let (tx, _rx) = broadcast::channel(16);

    loop {
        let (mut socket, addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            chat::handle_connection(addr, reader, writer, tx, rx)
                .await
                .expect("Failed to handle connection");
        });
    }
}
