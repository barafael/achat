use achat::{chat, init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use tokio::{net::TcpListener, sync::broadcast};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(addr) = args.console {
        init_console_subscriber(&addr);
    }

    let listener = TcpListener::bind(&args.address)
        .await
        .context(format!("Failed to bind on {}", &args.address))?;

    let (tx, _rx) = broadcast::channel(16);

    loop {
        if let Ok((mut socket, addr)) = listener.accept().await {
            println!("Received connection from {addr}");

            let tx = tx.clone();
            let rx = tx.subscribe();

            tokio::spawn(async move {
                let (reader, writer) = socket.split();
                chat::handle_connection(addr, reader, writer, tx, rx).await
            });
        }
    }
}
