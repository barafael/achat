use anyhow::Context;
use std::marker::Unpin;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

/// Uses [`tokio::io::copy`] to forward bytes.
///
/// # Termination
/// When EOF is found on the reader, terminate the future.
///
/// # Errors
/// Returns an error if forwarding failed.
pub async fn handle_connection<Reader, Writer>(
    mut reader: Reader,
    mut writer: Writer,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    tokio::io::copy(&mut reader, &mut writer)
        .await
        .map(|_n| ())
        .context("Forwarding reader to writer failed")
}

/// Manually loop, recv and send on `reader` and `writer`.
///
/// # Termination
/// In case the `reader` has no more bytes (`read_line` returned `Ok(0)`), terminate the future.
///
/// # Errors
/// Returns an error if something goes wrong while reading or writing.
pub async fn handle_connection_manually<Reader, Writer>(
    reader: Reader,
    mut writer: Writer,
) -> anyhow::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let mut line = String::new();
    let mut reader = BufReader::new(reader);

    loop {
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .context("Failed to read line")?;

        if bytes_read == 0 {
            break Ok(());
        }

        writer
            .write_all(line.as_bytes())
            .await
            .context("Failed to write")?;

        line.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio_test::io::Builder as Mock;

    #[tokio::test]
    async fn echo_works() {
        let writer = Mock::new().write(b"hello").build();
        let reader = Mock::new().read(b"hello").build();
        assert!(handle_connection(reader, writer).await.is_ok());
    }

    #[tokio::test]
    async fn manual_echo_works() {
        let writer = Mock::new().write(b"hello").build();
        let reader = Mock::new().read(b"hello").build();
        assert!(handle_connection_manually(reader, writer).await.is_ok());
    }
}
