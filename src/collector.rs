use anyhow::Context;
use std::collections::HashMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::{mpsc, oneshot},
};

/// A message sent from a client to the server.
#[derive(Debug)]
pub enum Message {
    /// Just some text.
    Text {
        /// Origin of message.
        sender: String,
        /// Message content.
        content: String,
    },

    /// Request for the current message report.
    Report {
        /// A oneshot channel for sending the reply.
        reply: oneshot::Sender<String>,
    },
}

/// Receive messages on reader. When receiving `"report"` or `"report\n"` or `"report\r\n"`,
/// send a report request, await the reply, and forward it on `writer`.
/// Else, just forward the message on the collection sender `tx`.
///
/// # Termination
/// In case the `reader` has no more bytes (`read_line` returned `Ok(0)`), terminate the future.
pub async fn handle_connection<Reader, Writer>(
    name: String,
    reader: Reader,
    mut writer: Writer,
    tx: mpsc::Sender<Message>,
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
            if is_report(&line) {
                let (sender, receiver) = oneshot::channel();
                let request = Message::Report { reply: sender };
                tx.send(request)
                    .await
                    .context("Failed to broadcast message from client")?;
                let report = receiver.await.context("Failed to fetch report")?;
                writer
                    .write_all(report.as_bytes())
                    .await
                    .context("Failed to forward report to client")?;
            } else {
                tx.send(Message::Text {
                    sender: name.clone(),
                    content: line.clone(),
                })
                .await
                .context("Failed to send text message to server")?;
            }
            line.clear();
        }
    }
}

/// If `line` is `"report"` or `"report\n"` or `"report\r\n"`, return `true`.
fn is_report(line: &str) -> bool {
    line == "report" || line == "report\n" || line == "report\r\n"
}

/// Receive messages on the given [`tokio::sync::mpsc::Receiver`].
/// On receiving a simple text message, just add it to the hashmap (the key being the source socket address).
/// On receiving a report message including a reply callback ([`tokio::sync::oneshot::Sender`]), serialize the hashmap, then send it on the callback.
///
/// # Termination
/// In case there are no more senders, terminate the future.
///
/// # Errors
/// Collection can fail if state serialization fails.
pub async fn collect(mut rx: mpsc::Receiver<Message>) -> anyhow::Result<()> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    loop {
        match rx.recv().await {
            Some(message) => match message {
                Message::Text { sender, content } => {
                    map.entry(sender).or_default().push(content);
                }
                Message::Report { reply } => {
                    if reply
                        .send(format!(
                            "{}\n",
                            serde_json::to_string_pretty(&map)
                                .context("Unable to serialize state")?,
                        ))
                        .is_err()
                    {
                        eprintln!("Failed to send report on client callback");
                    }
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
        let expected = [("test".to_string(), vec!["hello\r\n".to_string()])]
            .into_iter()
            .collect::<HashMap<String, Vec<String>>>();

        let writer = Mock::new()
            .write(format!("{}\n", serde_json::to_string_pretty(&expected).unwrap()).as_bytes())
            .build();
        let reader = Mock::new().read(b"hello\r\n").read(b"report\r\n").build();

        let (tx, rx) = mpsc::channel(16);

        let server = tokio::spawn(collect(rx));
        let client = tokio::spawn(handle_connection("test".to_string(), reader, writer, tx));

        tokio::join!(client, server).0.unwrap().unwrap();
    }
}
