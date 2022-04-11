use anyhow::Context;
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::{mpsc, oneshot},
};

#[derive(Debug)]
pub enum Message {
    Text(String),
    Report {
        //addr: Option<SocketAddr>,
        callback: oneshot::Sender<String>,
    },
}

pub async fn handle_connection<Reader, Writer>(
    addr: SocketAddr,
    reader: Reader,
    mut writer: Writer,
    tx: mpsc::Sender<(Message, SocketAddr)>,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let mut line = String::new();
    let mut reader = BufReader::new(reader);

    loop {
        if let Ok(bytes_read) = reader.read_line(&mut line).await {
            if bytes_read == 0 {
                break Ok(());
            }
            if line.as_str() == "report\r\n" {
                let (sender, receiver) = oneshot::channel();
                let request = Message::Report { callback: sender };
                tx.send((request, addr))
                    .await
                    .context("Failed to broadcast message from client")?;
                let report = receiver.await.unwrap();
                writer.write_all(report.as_bytes()).await.unwrap();
            } else {
                tx.send((Message::Text(line.clone()), addr)).await.unwrap();
            }
            line.clear();
        }
    }
}

pub async fn collect(mut rx: mpsc::Receiver<(Message, SocketAddr)>) -> anyhow::Result<()> {
    let mut map: HashMap<SocketAddr, Vec<String>> = HashMap::new();
    loop {
        match rx.recv().await {
            Some((message, source)) => match message {
                Message::Text(line) => {
                    map.entry(source).or_default().push(line);
                }
                Message::Report { callback } => {
                    callback.send(format!("{:#?}\n", map)).unwrap();
                }
            },
            None => {
                break Ok(());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_test::io::Builder as Mock;

    #[tokio::test]
    async fn collects_messages() {
        let expected = [(
            "127.0.0.3:8081".parse().unwrap(),
            vec!["hello\r\n".to_string()],
        )]
        .into_iter()
        .collect::<HashMap<SocketAddr, Vec<String>>>();

        let writer = Mock::new()
            .write(format!("{:#?}\n", expected).as_bytes())
            .build();
        let reader = Mock::new().read(b"hello\r\n").read(b"report\r\n").build();

        let (tx, rx) = mpsc::channel(16);

        let server = tokio::spawn(collect(rx));
        let client = tokio::spawn(handle_connection(
            "127.0.0.3:8081".parse().unwrap(),
            reader,
            writer,
            tx,
        ));

        tokio::try_join!(client).unwrap().0.unwrap();
        tokio::try_join!(server).unwrap().0.unwrap();
    }
}
