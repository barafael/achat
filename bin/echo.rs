use achat::{echo, init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if let Some(addr) = args.console {
        init_console_subscriber(&addr);
    }

    let listener = TcpListener::bind(&args.address)
        .await
        .context(format!("Failed to bind on {}", &args.address))?;

    loop {
        let (mut socket, _addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            echo::handle_connection(reader, writer)
                .await
                .expect("Failed to handle connection");
        });
    }
}
