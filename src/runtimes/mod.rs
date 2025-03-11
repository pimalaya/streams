//! Collection of stream runtimes.
//!
//! A runtime contains all the I/O logic, and is responsible for
//! processing [`Io`] requests emitted by [coroutines].
//!
//! If you miss a runtime matching your requirements, you can easily
//! implement your own by taking example on the existing ones. PRs are
//! welcomed!
//!
//! [`Io`]: crate::Io
//! [coroutines]: crate::coroutines

#[cfg(feature = "std")]
pub mod std;
#[cfg(feature = "tokio")]
pub mod tokio;
