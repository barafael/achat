use achat::echo;
use anyhow::Context;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("localhost:8080")
        .await
        .context("Failed to bind on localhost")?;

    loop {
        let (mut socket, _addr) = listener
            .accept()
            .await
            .context("Failed to accept on socket")?;

        tokio::spawn(async move {
            let (reader, writer) = socket.split();
            echo::handle_connection(reader, writer)
                .await
                .expect("Failed to handle connection.");
        });
    }
}
