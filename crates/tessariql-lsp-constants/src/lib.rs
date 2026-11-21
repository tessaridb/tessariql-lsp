//! Constants shared across the TessariQL language server crates.
//!
//! Every value here is a protocol or domain constant with its unit and reason
//! stated at the definition site — no magic numbers elsewhere.

/// The header an LSP message announces its byte length with, lowercase on the
/// wire (`content-length:`), compared case-insensitively per the spec.
pub const CONTENT_LENGTH_HEADER: &str = "content-length";

/// The media type of an LSP base-protocol message (`Content-Type` header).
pub const LSP_MEDIA_TYPE: &str = "application/vscode-jsonrpc; charset=utf-8";

pub mod docs;
pub mod language;

/// The LSP specification version this server implements (3.17).
pub const LSP_SPEC_VERSION: &str = "3.17";
