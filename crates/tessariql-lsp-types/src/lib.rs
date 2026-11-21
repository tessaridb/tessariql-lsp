//! Types shared across TessariQL language server crates.
//!
//! Value objects only — no I/O, no protocol framing.

pub mod document;
pub mod error;

pub use document::{DocumentId, DocumentText, DocumentVersion};
pub use error::{TypeError, TypeResult};
