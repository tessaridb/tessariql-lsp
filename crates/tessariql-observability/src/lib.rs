//! Tracing setup for the server binary.
//!
//! LSP rides stdio, so logs go to **stderr** — writing a log line to stdout
//! would corrupt the protocol stream.

mod tracing_setup;

pub use tracing_setup::init;
