use anyhow::Context;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::broadcast,
};
use tokio_util::sync::CancellationToken;

/// Monitor the `reader` and the `rx` for messages.
/// When receiving bytes on `reader`, forward them on the [`tokio::sync::broadcast::Sender`].
/// When receiving a message on `rx`, where the source socket address is not our own,
/// forward it on `writer` (else, discard it).
///
/// # Termination
/// If an error or `None` is encountered, the future terminates.
/// If EOF is signalled on `reader` by `Ok(0)`, the future terminates.
/// If the text read from `reader` is `"quit"` or `"quit\n"` or `"quit\r\n"`, the future terminates.
/// If the text read from `reader` is `"call it a day"` or `"call it a day\n"` or `"call it a day\r\n"`,
/// the [`tokio_util::sync::CancellationToken`] is triggered.
/// If the `token` is cancelled somewhere else, the future terminates.
pub async fn handle_connection<Reader, Writer>(
    addr: SocketAddr,
    reader: Reader,
    mut writer: Writer,
    tx: broadcast::Sender<(String, SocketAddr)>,
    mut rx: broadcast::Receiver<(String, SocketAddr)>,
    token: CancellationToken,
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
                    break Ok::<(), anyhow::Error>(()); // EOF detected.
                }
                if is_termination_message(&line) {
                    token.cancel();
                }
                if line == "quit" || line == "quit\r\n" {
                    break Ok(());
                }
                tx.send((format!("{addr}: {line}"), addr)).context("Failed to broadcast message")?;
            },
            Ok((message, source)) = rx.recv() => {
                if source == addr {
                    continue;
                }
                writer.write_all(message.as_bytes()).await.context("Failed to forward message")?;
            }
            _ = token.cancelled() => {
                break Ok(());
            },
            else => {
                break Ok(());
            }
        }
    }
    .context("Client failed to handle messages")?;
    // Flush data in writer.
    writer
        .shutdown()
        .await
        .context("Unable to shut down client writer")
}

/// If the `line` is `"call it a day"` or `"call it a day\n"` or `"call it a day\r\n"`, return `true`.
fn is_termination_message(line: &str) -> bool {
    line == "call it a day" || line == "call it a day\n" || line == "call it a day\r\n"
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

        let token = CancellationToken::new();

        let handle = tokio::spawn(handle_connection(
            "127.0.0.3:8081".parse().unwrap(),
            reader,
            writer,
            tx.clone(),
            tx.subscribe(),
            token,
        ));

        let (message, socket) = rx.recv().await.unwrap();
        assert_eq!(
            (message, socket),
            (
                "127.0.0.3:8081: hello".to_string(),
                "127.0.0.3:8081".parse().unwrap()
            )
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

        let token = CancellationToken::new();

        let handle = tokio::spawn(handle_connection(
            "127.0.0.3:8081".parse().unwrap(),
            reader,
            writer,
            tx.clone(),
            tx.subscribe(),
            token,
        ));

        tx.send((
            "how's it going".to_string(),
            "127.0.0.1:1234".parse().unwrap(),
        ))
        .unwrap();

        tokio::join!(handle).0.unwrap().unwrap();
    }
}
