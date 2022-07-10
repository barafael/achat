use achat::{init_console_subscriber, Args};
use anyhow::Context;
use klask::Settings;
use std::str;
use tokio::{io::AsyncReadExt, net::TcpListener};

fn main() {
    klask::run_derived::<Args, _>(Settings::default(), start_async);
}

fn start_async(args: Args) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { dump_server(args).await.unwrap() });
}

async fn dump_server(args: Args) -> anyhow::Result<()> {
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
