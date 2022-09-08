use achat::Args;
use clap::Parser;
use tokio::io::{copy, stdin, stdout, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::pin;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut stream = TcpStream::connect(args.address).await?;
    let (reader, writer) = stream.split();
    let (stdin, stdout) = (stdin(), stdout());
    pin!(reader, writer, stdin, stdout);
    tokio::try_join!(
        copy(&mut stdin, &mut writer),
        copy(&mut reader, &mut stdout),
    )?;
    println!("Done");
    stdout.flush().await.unwrap();
    Ok(())
}
