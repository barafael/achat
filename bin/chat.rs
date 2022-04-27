use achat::{chat, init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::{io::AsyncWriteExt, net::TcpListener, sync::broadcast};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(addr) = args.console {
        init_console_subscriber(addr);
    }

    let listener = TcpListener::bind(&args.address)
        .await
        .context(format!("Failed to bind on {}", &args.address))?;

    let (tx, _rx) = broadcast::channel(16);

    loop {
        let (socket, addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        let mut socket = BufReader::new(socket);

        let tx = tx.clone();
        let rx = tx.subscribe();

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
                chat::handle_connection(name.clone(), addr, reader, writer, tx, rx)
                    .await
                    .expect("Failed to handle connection");
            });
    }
}
