use anyhow::Context;
use std::{net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::{broadcast, watch},
};

pub async fn handle_connection<Reader, Writer>(
    addr: SocketAddr,
    reader: Reader,
    mut writer: Writer,
    tx: broadcast::Sender<(String, SocketAddr)>,
    mut rx: broadcast::Receiver<(String, SocketAddr)>,
    mut topic_rx: watch::Receiver<String>,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let mut line = String::new();
    let mut reader = BufReader::new(reader);

    loop {
        tokio::select! {
            Ok(bytes_read) = reader.read_line(&mut line) => {
                if bytes_read == 0 {
                    break Ok(());
                }
                tx.send((line.clone(), addr)).context("Failed to broadcast message from client")?;
            },
            Ok((message, source)) = rx.recv() => {
                if source == addr {
                    continue;
                }
                writer.write_all(message.as_bytes()).await.context("Failed to write message to client")?;
            },
            Ok(()) = topic_rx.changed() => {
                writer.write_all(
                    format!("topic changed: {}\n", *topic_rx.borrow()).as_bytes()).await.context("Unable to forward topic change")?;
            }
            else => {
                break Ok(());
            }
        }
    }
}

pub async fn announce_uptime(
    topic: watch::Sender<String>,
    duration: Duration,
) -> anyhow::Result<()> {
    let mut i = 0u64;
    let mut interval = tokio::time::interval(duration);
    loop {
        interval.tick().await;
        if let Err(_e) = topic.send(format!("Up for {}s.", i * duration.as_secs())) {
            // receivers dropped.
            break Ok(());
        }
        i += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use tokio_test::io::Builder as Mock;

    #[tokio::test]
    async fn broadcasts_message() {
        let writer = Mock::new().build();
        let reader = Mock::new().read(b"hello").build();

        let (tx, mut rx) = broadcast::channel(16);
        let (_topic_tx, topic_rx) = watch::channel("Chat topic".to_string());

        let handle = tokio::spawn(handle_connection(
            "127.0.0.3:8081".parse().unwrap(),
            reader,
            writer,
            tx.clone(),
            tx.subscribe(),
            topic_rx,
        ));

        let (message, socket) = rx.recv().await.unwrap();
        assert_eq!(
            (message, socket),
            ("hello".to_string(), "127.0.0.3:8081".parse().unwrap())
        );

        tokio::join!(handle).0.unwrap().unwrap();
    }

    #[tokio::test]
    async fn receives_message() {
        tokio::time::pause();
        let writer = Mock::new().write(b"how's it going").build();
        // give the writer time to be written before reader is read, returning 'no data', ending handle_connection.
        let reader = Mock::new().wait(Duration::from_secs(1)).build();

        let (tx, _rx) = broadcast::channel(1);
        let (_topic_tx, topic_rx) = watch::channel("Chat topic".to_string());

        let handle = tokio::spawn(handle_connection(
            "127.0.0.3:8081".parse().unwrap(),
            reader,
            writer,
            tx.clone(),
            tx.subscribe(),
            topic_rx,
        ));

        tx.send((
            "how's it going".to_string(),
            "127.0.0.1:1234".parse().unwrap(),
        ))
        .unwrap();

        tokio::join!(handle).0.unwrap().unwrap();
    }

    #[tokio::test]
    async fn announces_time() {
        tokio::time::pause();
        let (tx, mut rx) = watch::channel("initial".to_string());
        let duration = Duration::from_secs(1);

        let handle = tokio::spawn(announce_uptime(tx, duration));

        rx.changed().await.unwrap();
        let reply = rx.borrow().clone();
        assert_eq!("Up for 0s.", reply);

        rx.changed().await.unwrap();
        let reply = rx.borrow().clone();
        assert_eq!("Up for 1s.", reply);

        rx.changed().await.unwrap();
        let reply = rx.borrow().clone();
        assert_eq!("Up for 2s.", reply);

        drop(rx);
        tokio::join!(handle).0.unwrap().unwrap();
    }

    #[tokio::test]
    async fn forwards_announcements_to_clients() {
        tokio::time::pause();
        let writer = Mock::new()
            .write(b"topic changed: hello\n")
            .write(b"topic changed: i'm\n")
            .write(b"topic changed: a\n")
            .write(b"topic changed: teapot\n")
            .build();
        let reader = Mock::new().wait(Duration::from_secs(1)).build();

        let (tx, _rx) = broadcast::channel(16);
        let (topic_tx, topic_rx) = watch::channel("Discarded initial topic".to_string());

        let handle = tokio::spawn(handle_connection(
            "127.0.0.3:8081".parse().unwrap(),
            reader,
            writer,
            tx.clone(),
            tx.subscribe(),
            topic_rx,
        ));

        topic_tx.send("hello".to_string()).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        topic_tx.send("i'm".to_string()).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        topic_tx.send("a".to_string()).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        topic_tx.send("teapot".to_string()).unwrap();

        tokio::join!(handle).0.unwrap().unwrap();
    }
}
