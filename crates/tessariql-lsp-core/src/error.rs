use thiserror::Error;

use tessariql_lsp_types::TypeError;

/// Errors of the domain crate.
#[derive(Debug, Error)]
pub enum CoreError {
    /// A document operation violated the shared type contract.
    #[error(transparent)]
    Types(#[from] TypeError),
}

/// Result alias for this crate.
pub type CoreResult<T> = Result<T, CoreError>;
