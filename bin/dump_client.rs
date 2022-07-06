use achat::{init_console_subscriber, Args};
use anyhow::Context;
use clap::Parser;
use std::str;
use tokio::io::AsyncReadExt;
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
        if let Ok((mut socket, addr)) = listener.accept().await {
            println!("Received connection from {addr}");

            tokio::spawn(async move {
                let (mut reader, _) = socket.split();
                let mut buffer = [0; 1024];
                loop {
                    let n = reader.read(&mut buffer).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    match str::from_utf8(&buffer[..n]) {
                        Ok(s) => println!("Received {n} bytes: {s:#?}"),
                        Err(e) => println!("Received {n} bytes of wrong UTF8 data: {e:#?}"),
                    }
                }
            });
        }
    }
}
