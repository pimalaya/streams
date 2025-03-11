//! Collection of I/O-free, resumable and composable stream state
//! machines.
//!
//! Coroutines emit [`Io`] requests that need to be processed by
//! [runtimes] in order to continue their progression.
//!
//! [`Io`]: crate::Io
//! [runtimes]: crate::runtimes

mod read;
mod write;

#[doc(inline)]
pub use self::{read::Read, write::Write};
