use thiserror::Error;

/// Errors of the shared type crate.
#[derive(Debug, Error)]
pub enum TypeError {
    /// A document version would move backwards; LSP versions are monotonic.
    #[error("document version regressed: {from} -> {to}")]
    VersionRegressed { from: i32, to: i32 },
}

/// Result alias for this crate.
pub type TypeResult<T> = Result<T, TypeError>;
