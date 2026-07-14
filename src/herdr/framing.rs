use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

pub async fn write_json_line<W, T>(writer: &mut W, value: &T) -> Result<(), FramingError>
where
    W: AsyncWrite + Unpin,
    T: Serialize + ?Sized,
{
    let encoded = serde_json::to_vec(value)?;
    writer.write_all(&encoded).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_json_line<R, T>(reader: &mut R) -> Result<T, FramingError>
where
    R: AsyncBufRead + Unpin,
    T: DeserializeOwned,
{
    read_optional_json_line(reader)
        .await?
        .ok_or(FramingError::Eof)
}

pub async fn read_optional_json_line<R, T>(reader: &mut R) -> Result<Option<T>, FramingError>
where
    R: AsyncBufRead + Unpin,
    T: DeserializeOwned,
{
    let mut line = String::new();
    let read = reader.read_line(&mut line).await?;
    if read == 0 {
        return Ok(None);
    }
    if line.trim().is_empty() {
        return Err(FramingError::EmptyLine);
    }
    serde_json::from_str(&line)
        .map(Some)
        .map_err(FramingError::Json)
}

#[derive(Debug, Error)]
pub enum FramingError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON line: {0}")]
    Json(#[from] serde_json::Error),
    #[error("received an empty JSON line")]
    EmptyLine,
    #[error("connection closed before a JSON line was received")]
    Eof,
}
