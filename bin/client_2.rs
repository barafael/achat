use achat::Args;
use clap::Parser;
use tokio::io::{copy, stdin, stdout};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut stream = TcpStream::connect(args.address).await?;
    let (mut reader, mut writer) = stream.split();
    let (mut stdin, mut stdout) = (stdin(), stdout());
    tokio::try_join!(
        copy(&mut stdin, &mut writer),
        copy(&mut reader, &mut stdout),
    )?;
    Ok(())
}
