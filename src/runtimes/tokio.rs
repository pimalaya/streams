//! Module dedicated to the Tokio-based, async runtime.

use std::io;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{Io, Output};

/// The main runtime I/O handler.
///
/// This handler makes use of the [`tokio::io`] module as well as
/// standard module [`std::io`] to process stream [`Io`].
pub async fn handle(stream: impl AsyncRead + AsyncWrite + Unpin, io: Io) -> io::Result<Io> {
    match io {
        Io::UnavailableInput => {
            let kind = io::ErrorKind::InvalidInput;
            Err(io::Error::new(kind, "input has already been used"))
        }
        Io::UnexpectedInput(io) => {
            let kind = io::ErrorKind::InvalidInput;
            Err(io::Error::new(kind, format!("unexpected input: {io:?}")))
        }
        Io::Read(io) => read(stream, io).await,
        Io::Write(io) => write(stream, io).await,
    }
}

pub async fn read(
    mut stream: impl AsyncRead + Unpin,
    input: Result<Output, Vec<u8>>,
) -> io::Result<Io> {
    let Err(mut buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "missing read buffer"));
    };

    let bytes_count = stream.read(&mut buffer).await?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Read(Ok(output)))
}

pub async fn write(
    mut stream: impl AsyncWrite + Unpin,
    input: Result<Output, Vec<u8>>,
) -> io::Result<Io> {
    let Err(buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "missing write bytes"));
    };

    let bytes_count = stream.write(&buffer).await?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Write(Ok(output)))
}
