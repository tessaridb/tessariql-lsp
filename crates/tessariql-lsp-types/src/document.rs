//! Document identity and content types.

use crate::error::{TypeError, TypeResult};

/// Stable identity of an open document, keyed by its URI in the editor session.
///
/// The URI is owned (editors send `file://` and `untitled:` schemes alike); the
/// server never resolves or touches the filesystem, it only correlates what the
/// client sent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DocumentId(String);

impl DocumentId {
    /// Builds an id from the URI exactly as the client reported it.
    #[must_use]
    pub fn from_uri(uri: &str) -> Self {
        Self(uri.to_owned())
    }

    /// The URI as reported by the client.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Monotonic version of a document, as assigned by the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentVersion(i32);

impl DocumentVersion {
    /// First version of a freshly opened document.
    pub const INITIAL: Self = Self(0);

    /// Builds a version from the LSP `version` field.
    #[must_use]
    pub fn from_lsp(version: i32) -> Self {
        Self(version)
    }

    /// The raw LSP value.
    #[must_use]
    pub fn as_i32(self) -> i32 {
        self.0
    }

    /// Returns `VersionRegressed` when `next` does not advance this version.
    /// LSP guarantees monotonically increasing versions; a regression means a
    /// lost update on the client side and must surface, not be silently applied.
    pub fn check_advance(self, next: Self) -> TypeResult<()> {
        if next.0 <= self.0 {
            return Err(TypeError::VersionRegressed {
                from: self.0,
                to: next.0,
            });
        }
        Ok(())
    }
}

/// Full text of an open document. Owned because LSP full sync replaces the
/// whole content; incremental sync lands with the grammar milestone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentText(String);

impl DocumentText {
    /// Builds the text from an opened or changed document.
    #[must_use]
    pub fn new(text: String) -> Self {
        Self(text)
    }

    /// The current content.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Length in bytes (LSP positions are UTF-16; conversion arrives with the
    /// line index in the grammar milestone).
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_advances_monotonically() {
        let v = DocumentVersion::INITIAL;
        assert!(v.check_advance(DocumentVersion(1)).is_ok());
        let err = v.check_advance(DocumentVersion::INITIAL).unwrap_err();
        assert!(err.to_string().contains("regressed"));
    }

    #[test]
    fn id_keeps_the_uri_verbatim() {
        let id = DocumentId::from_uri("untitled:Untitled-1");
        assert_eq!(id.as_str(), "untitled:Untitled-1");
    }
}
