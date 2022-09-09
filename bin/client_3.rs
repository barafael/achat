use achat::Args;
use clap::Parser;
use readwrite::ReadWriteTokio;
use tokio::io::{copy_bidirectional, stdin, stdout};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut stream = TcpStream::connect(args.address).await?;
    let mut rw = ReadWriteTokio::new(stdin(), stdout());
    tokio::try_join!(copy_bidirectional(&mut rw, &mut stream))?;
    Ok(())
}
