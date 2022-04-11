use achat::collector;
use anyhow::Context;
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .context("Failed to bind on localhost")?;

    let (tx, rx) = mpsc::channel(16);

    let _handle = tokio::spawn(collector::collect(rx));

    loop {
        let (mut socket, addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        let tx = tx.clone();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            collector::handle_connection(addr, reader, writer, tx)
                .await
                .expect("Failed to handle connection");
        });
    }
}
