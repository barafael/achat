use achat::{collector, init_console_subscriber, Arguments};
use anyhow::Context;
use clap::Parser;
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    if let Some(addr) = args.console {
        init_console_subscriber(addr);
    }

    let listener = TcpListener::bind(&args.address)
        .await
        .context(format!("Failed to bind on {}", &args.address))?;

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
            collector::handle_connection(addr.to_string(), reader, writer, tx)
                .await
                .expect("Failed to handle connection");
        });
    }
}
