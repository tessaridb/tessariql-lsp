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

/// Environment variables the infrastructure crate reads (the feature flag
/// and its credentials). The password arrives from the environment only.
pub mod environment {
    /// `host:port` of the node; empty or absent = catalog features off.
    pub const NODE_ENV: &str = "TESSARIQL_LSP_NODE";
    /// User to sign in as; both this and the password, or neither.
    pub const USER_ENV: &str = "TESSARIQL_LSP_USER";
    /// The password; never an argument, never logged.
    pub const PASSWORD_ENV: &str = "TESSARIQL_LSP_PASSWORD";
    /// Namespace the catalog reads.
    pub const NAMESPACE_ENV: &str = "TESSARIQL_LSP_NAMESPACE";
    /// Database the catalog reads.
    pub const DATABASE_ENV: &str = "TESSARIQL_LSP_DATABASE";
}
