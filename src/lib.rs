#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod coroutines;
mod io;
pub mod runtimes;

#[doc(inline)]
pub use self::io::{Io, Output};
