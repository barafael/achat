use achat::chat;
use anyhow::Context;
use clap::Parser;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::{net::TcpListener, sync::broadcast};

/// Command Line Arguments
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Adress to listen on
    #[clap(short, long)]
    address: String,

    /// Address to publish console events on
    #[clap(short, long)]
    console: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    dbg!(&args);

    if let Some(console) = args.console {
        let console: SocketAddr = console.parse().unwrap();
        console_subscriber::ConsoleLayer::builder()
            .retention(Duration::from_secs(60))
            .server_addr(console)
            .init();
    }

    let listener = TcpListener::bind(args.address)
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
