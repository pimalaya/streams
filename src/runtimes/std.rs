//! Module dedicated to the standard, blocking runtime.

use std::io::{self, Read, Write};

use crate::{Io, Output};

/// The main runtime I/O handler.
///
/// This handler makes use of standard modules [`std::io`] to process
/// stream [`Io`].
pub fn handle(stream: impl Read + Write, io: Io) -> io::Result<Io> {
    match io {
        Io::UnavailableInput => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "input has already been used",
        )),
        Io::UnexpectedInput(io) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unexpected input: {io:?}"),
        )),
        Io::Read(io) => read(stream, io),
        Io::Write(io) => write(stream, io),
    }
}

pub fn read(mut stream: impl Read, input: Result<Output, Vec<u8>>) -> io::Result<Io> {
    let Err(mut buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "missing read buffer"));
    };

    let bytes_count = stream.read(&mut buffer)?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Read(Ok(output)))
}

pub fn write(mut stream: impl Write, input: Result<Output, Vec<u8>>) -> io::Result<Io> {
    let Err(buffer) = input else {
        let kind = io::ErrorKind::InvalidInput;
        return Err(io::Error::new(kind, "missing write bytes"));
    };

    let bytes_count = stream.write(&buffer)?;

    let output = Output {
        buffer,
        bytes_count,
    };

    Ok(Io::Write(Ok(output)))
}
