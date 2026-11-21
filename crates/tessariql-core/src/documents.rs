//! The registry of documents open in the editor session.
//!
//! One entry per `textDocument/didOpen`, replaced on `didChange`, removed on
//! `didClose`. Concurrent across LSP handler tasks; every method is a short
//! synchronous critical section and no guard ever crosses an `.await` — the
//! adapter calls, collects, drops the guard, and only then awaits.

use std::sync::Arc;

use dashmap::DashMap;
use tessariql_types::{DocumentId, DocumentText, DocumentVersion};
use tracing::{debug, instrument};

use crate::error::CoreResult;

/// An open document: latest accepted text and the version it belongs to.
#[derive(Debug, Clone)]
pub struct OpenDocument {
    version: DocumentVersion,
    text: DocumentText,
}

impl OpenDocument {
    /// The version of the text the editor currently holds.
    #[must_use]
    pub fn version(&self) -> DocumentVersion {
        self.version
    }

    /// The current content.
    #[must_use]
    pub fn text(&self) -> &DocumentText {
        &self.text
    }
}

/// Shared store of open documents.
#[derive(Debug, Default)]
pub struct Documents {
    inner: DashMap<DocumentId, OpenDocument>,
}

impl Documents {
    /// A fresh, empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Wraps the registry for sharing across handler tasks.
    #[must_use]
    pub fn shared(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// Opens a document. `didOpen` fires once per document; a repeat open is a
    /// client contract violation and is ignored rather than silently
    /// overwriting live state. One shard acquisition via the entry API — no
    /// check-then-act between two locks.
    #[instrument(skip(self, text), fields(uri = %id.as_str()))]
    pub fn open(&self, id: DocumentId, version: DocumentVersion, text: DocumentText) {
        match self.inner.entry(id) {
            dashmap::mapref::entry::Entry::Vacant(slot) => {
                slot.insert(OpenDocument { version, text });
            }
            dashmap::mapref::entry::Entry::Occupied(_) => {
                debug!("didOpen for an already-open document; ignoring");
            }
        }
    }

    /// Replaces the content on `didChange`, enforcing the monotonic-version
    /// contract so a lost update surfaces instead of clobbering newer text.
    /// A change for a document that is not open is ignored (the close raced
    /// the change); closing is idempotent, changing a closed document is not
    /// an error either.
    #[instrument(skip(self, text), fields(uri = %id.as_str()))]
    pub fn change(
        &self,
        id: &DocumentId,
        version: DocumentVersion,
        text: DocumentText,
    ) -> CoreResult<()> {
        let Some(mut entry) = self.inner.get_mut(id) else {
            debug!("didChange for a closed document; ignoring");
            return Ok(());
        };
        entry.version.check_advance(version)?;
        entry.text = text;
        entry.version = version;
        Ok(())
    }

    /// Removes the document on `didClose`. Closing an unknown id is a no-op —
    /// close is idempotent by nature.
    #[instrument(skip(self), fields(uri = %id.as_str()))]
    pub fn close(&self, id: &DocumentId) {
        self.inner.remove(id);
    }

    /// Snapshot of the current content, for handlers that only read. The clone
    /// is deliberate: the guard never leaves this function, so no shard lock
    /// can be held across a caller's `.await`.
    #[must_use]
    pub fn get(&self, id: &DocumentId) -> Option<OpenDocument> {
        self.inner.get(id).map(|entry| entry.clone())
    }

    /// Number of open documents (diagnostics/tests). Visits every shard —
    /// never on a per-request path.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Whether no document is open.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(uri: &str, version: i32, text: &str) -> (DocumentId, DocumentVersion, DocumentText) {
        (
            DocumentId::from_uri(uri),
            DocumentVersion::from_lsp(version),
            DocumentText::new(text.to_owned()),
        )
    }

    #[test]
    fn open_change_close_cycle() {
        let docs = Documents::new();
        let (id, version, text) = doc("file:///a.tessariql", 1, "SELECT 1;");
        docs.open(id.clone(), version, text);
        assert_eq!(docs.len(), 1);

        let (_, version2, text2) = doc("file:///a.tessariql", 2, "SELECT 2;");
        docs.change(&id, version2, text2).unwrap();
        let current = docs.get(&id).unwrap();
        assert_eq!(current.text().as_str(), "SELECT 2;");
        assert_eq!(current.version().as_i32(), 2);

        docs.close(&id);
        assert!(docs.is_empty());
    }

    #[test]
    fn regression_is_refused_and_newer_content_survives() {
        let docs = Documents::new();
        let (id, version, text) = doc("file:///a.tessariql", 5, "SELECT 1;");
        docs.open(id.clone(), version, text);

        let (_, older, older_text) = doc("file:///a.tessariql", 4, "stale");
        let result = docs.change(&id, older, older_text);
        assert!(result.is_err());
        assert_eq!(docs.get(&id).unwrap().version().as_i32(), 5);
    }

    #[test]
    fn repeat_open_does_not_overwrite() {
        let docs = Documents::new();
        let (id, version, text) = doc("untitled:Untitled-1", 1, "SELECT 1;");
        docs.open(id.clone(), version, text);
        let (_, v2, t2) = doc("untitled:Untitled-1", 9, "replaced?");
        docs.open(id.clone(), v2, t2);
        assert_eq!(docs.get(&id).unwrap().version().as_i32(), 1);
    }
}
