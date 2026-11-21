//! Domain logic of the TessariQL language server.
//!
//! Everything here is synchronous and I/O-free: the LSP adapter calls into the
//! domain, never the other way around.

pub mod completion;
pub mod documents;
pub mod error;
pub mod hover;

pub use completion::{Completion, CompletionKind, complete};
pub use documents::{Documents, OpenDocument};
pub use error::{CoreError, CoreResult};
pub use hover::hover;
