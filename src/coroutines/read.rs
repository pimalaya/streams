//! Module dedicated to the [`Read`] I/O-free coroutine.

use crate::{Io, Output};

/// I/O-free coroutine for reading bytes into a buffer.
#[derive(Debug)]
pub struct Read {
    buffer: Option<Vec<u8>>,
}

impl Read {
    /// Creates a new coroutine from the given buffer mutable
    /// reference.
    pub fn new(buffer: Vec<u8>) -> Self {
        let buffer = Some(buffer);
        Self { buffer }
    }

    pub fn set_buffer(&mut self, buffer: Vec<u8>) {
        self.buffer = Some(buffer);
    }

    /// Makes the coroutine progress.
    pub fn resume(&mut self, input: Option<Io>) -> Result<Output, Io> {
        let Some(input) = input else {
            return Err(match self.buffer.take() {
                Some(buffer) => Io::Read(Err(buffer)),
                None => Io::UnavailableInput,
            });
        };

        let Io::Read(output) = input else {
            return Err(Io::UnexpectedInput(Box::new(input)));
        };

        match output {
            Ok(output) => Ok(output),
            Err(io) => Err(Io::Read(Err(io))),
        }
    }
}

impl Default for Read {
    fn default() -> Self {
        Self {
            buffer: Some(vec![0; 1024]),
        }
    }
}
