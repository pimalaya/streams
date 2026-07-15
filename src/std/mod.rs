//! Blocking std runtime layer.
//!
//! Hosts the std-blocking transport; the module is named after its
//! runtime on purpose, so a future async runtime gains a sibling
//! module (tokio) next to it.

pub mod stream;
