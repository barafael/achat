use achat::Arguments;
use bytes::BytesMut;
use clap::Parser;
use futures::SinkExt;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Arguments::parse();

    let stdin = FramedRead::new(tokio::io::stdin(), BytesCodec::new());
    let mut stdin = stdin.map(|i| i.map(BytesMut::freeze));
    let mut stdout = FramedWrite::new(tokio::io::stdout(), BytesCodec::new());

    let mut stream = TcpStream::connect(args.address).await?;
    let (reader, writer) = stream.split();
    let mut sink = FramedWrite::new(writer, BytesCodec::new());
    let mut stream = FramedRead::new(reader, BytesCodec::new());

    loop {
        tokio::select! {
            msg = stream.next() => {
                if let Some(Ok(msg)) = msg {
                    stdout.send(msg).await.unwrap();
                } else {
                    break
                }
            },
            input = stdin.next() => {
                if let Some(Ok(input)) = input {
                    sink.send(input).await.unwrap();
                } else {
                    break;
                }
            }
        }
    }
    println!("Done");
    tokio::io::stdout().flush().await.unwrap();
    Ok(())
}
