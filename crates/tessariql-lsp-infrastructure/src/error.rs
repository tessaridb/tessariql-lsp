//! Errors of the infrastructure crate: connection, refusal, and the catalog
//! shapes that do not say what this build expected.

use tessaridb_client::Error as ClientError;

/// What can go wrong reaching the node or reading its catalog. A node
/// refusal travels as the node's own message — the editor shows what the
/// store said, not a paraphrase of it.
#[derive(Debug, thiserror::Error)]
pub enum InfraError {
    /// The wire protocol or the transport failed underneath the client.
    #[error("node connection: {0}")]
    Client(#[from] ClientError),
    /// The node answered, but not with the object shape the catalog reader
    /// knows — a newer node, most likely; the safe answer is "no catalog".
    #[error("catalog answer was not an object")]
    NotAnObject,
    /// A statement ran but every answer is an error string to show as-is.
    #[error("{0}")]
    Refused(String),
}

impl InfraError {
    /// The refusal text, when the error is the node's own words.
    #[must_use]
    pub fn refusal_text(&self) -> Option<&str> {
        match self {
            Self::Refused(text) => Some(text),
            _ => None,
        }
    }
}

pub type InfraResult<T> = Result<T, InfraError>;
