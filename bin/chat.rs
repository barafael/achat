use achat::{chat, init_console_subscriber, Args};
use anyhow::{Context, Ok};
use clap::Parser;
use tokio::{net::TcpListener, sync::broadcast};
use tokio_util::sync::CancellationToken;

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

    let mut handles = Vec::new();

    let token = CancellationToken::new();
    loop {
        let token = token.clone();
        tokio::select! {
            _ = token.cancelled() => {
                break Ok(());
            },
            listen = listener.accept() => {
                let (mut socket, addr) = listen.context("Failed to accept on socket")?;
                let tx = tx.clone();
                let rx = tx.subscribe();

                let h = tokio::spawn(async move {
                    let (reader, writer) = socket.split();
                    chat::handle_connection(addr, reader, writer, tx, rx, token.clone())
                        .await
                        .context("Failed to handle connection")?;
                    Ok(())
                });
                handles.push(h);
            }
        };
    }
    .context("Failed to run the accept loop")?;
    futures::future::try_join_all(handles)
        .await
        .context("Unable to join client tasks")?
        .into_iter()
        .collect::<Result<(), _>>()
        .context("Error in client task")
}
