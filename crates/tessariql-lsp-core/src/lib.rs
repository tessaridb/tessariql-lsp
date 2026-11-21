//! Domain logic of the TessariQL language server.
//!
//! Everything here is synchronous and I/O-free: the LSP adapter calls into the
//! domain, never the other way around.

pub mod documents;
pub mod error;

pub use documents::{Documents, OpenDocument};
pub use error::{CoreError, CoreResult};
