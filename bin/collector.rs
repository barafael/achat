use achat::{collector, get_name, init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(addr) = args.console {
        init_console_subscriber(addr);
    }

    let listener = TcpListener::bind(&args.address)
        .await
        .context(format!("Failed to bind on {}", &args.address))?;

    let (tx, rx) = mpsc::channel(16);

    let _handle = tokio::spawn(collector::collect(rx));

    loop {
        let (mut socket, _addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        let name = get_name(&mut socket).await.context("Failed to get name")?;

        let tx = tx.clone();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            collector::handle_connection(name, reader, writer, tx)
                .await
                .expect("Failed to handle connection");
        });
    }
}
