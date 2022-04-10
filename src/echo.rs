use anyhow::Context;
use std::marker::Unpin;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};

pub async fn handle_connection<Reader, Writer>(
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
    use tokio_test::io::Builder as Mock;

    use super::*;

    #[tokio::test]
    async fn echo_works() {
        let writer = Mock::new().write(b"hello").build();
        let reader = Mock::new().read(b"hello").build();
        assert!(handle_connection(reader, writer).await.is_ok());
    }
}
