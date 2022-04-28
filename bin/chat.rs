use achat::{chat, get_name, init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use tokio::io::BufReader;
use tokio::{net::TcpListener, sync::broadcast};

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

        let name = get_name(&mut socket).await.context("Failed to get name")?;

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
