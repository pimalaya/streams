//! Module dedicated to the [`Write`] I/O-free coroutine.

use crate::{Io, Output};

/// I/O-free coroutine for writing bytes into a stream.
#[derive(Debug, Default)]
pub struct Write {
    bytes: Option<Vec<u8>>,
}

impl Write {
    /// Creates a new coroutine from the given buffer reference.
    pub fn new(bytes: impl IntoIterator<Item = u8>) -> Self {
        let bytes = Some(bytes.into_iter().collect());
        Self { bytes }
    }

    pub fn set_bytes(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.bytes = Some(bytes.into_iter().collect());
    }

    pub fn with_bytes(mut self, bytes: impl IntoIterator<Item = u8>) -> Self {
        self.set_bytes(bytes);
        self
    }

    pub fn enqueue_bytes(&mut self, bytes: impl IntoIterator<Item = u8>) {
        match &mut self.bytes {
            Some(prev_bytes) => prev_bytes.extend(bytes),
            None => self.set_bytes(bytes),
        }
    }

    /// Makes the coroutine progress.
    pub fn resume(&mut self, input: Option<Io>) -> Result<Output, Io> {
        let Some(input) = input else {
            return Err(match self.bytes.take() {
                Some(bytes) => Io::Write(Err(bytes)),
                None => Io::UnavailableInput,
            });
        };

        let Io::Write(output) = input else {
            return Err(Io::UnexpectedInput(Box::new(input)));
        };

        match output {
            Ok(output) => Ok(output),
            Err(io) => Err(Io::Write(Err(io))),
        }
    }
}
