use achat::chat_with_announce::{self, announce_uptime};
use achat::{init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::{
    net::TcpListener,
    sync::{broadcast, watch},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(console) = args.console {
        init_console_subscriber(console);
    }

    let listener = TcpListener::bind(&args.address)
        .await
        .context(format!("Failed to bind on {}", &args.address))?;

    let (tx, _rx) = broadcast::channel(16);
    let (topic_tx, topic_rx) = watch::channel("Chat topic".to_string());

    tokio::spawn(announce_uptime(topic_tx, Duration::from_secs(10)));

    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        let mut socket = BufReader::new(socket);

        let tx = tx.clone();
        let rx = tx.subscribe();
        let topic_rx = topic_rx.clone();

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

        tokio::task::Builder::new()
            .name(name.clone().as_str())
            .spawn(async move {
                let mut socket = socket.into_inner();
                let (reader, writer) = socket.split();
                chat_with_announce::handle_connection(
                    name.clone(),
                    addr,
                    reader,
                    writer,
                    tx,
                    rx,
                    topic_rx,
                )
                .await
                .expect("Failed to handle connection");
            });
    }
}
