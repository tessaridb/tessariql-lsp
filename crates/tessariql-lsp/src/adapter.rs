//! The LSP adapter: protocol calls in, domain calls, protocol answers out.
//!
//! Everything here is translation. State transitions live in
//! [`tessariql_lsp_core`]; shared-state discipline: every domain call is a short
//! synchronous section, the result is collected, and no guard is held across
//! an `.await` in this file.

use std::sync::Arc;

use tessariql_lsp_core::Documents;
use tessariql_lsp_types::{DocumentId, DocumentText, DocumentVersion};
use tower_lsp_server::jsonrpc::Result as LspResult;
use tower_lsp_server::ls_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, MessageType, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tower_lsp_server::{Client, LanguageServer, LspService};
use tracing::{debug, instrument, warn};

/// Handler state: the editor-facing client channel and the open documents.
#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: Arc<Documents>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                // Full sync for now: incremental sync arrives with the grammar
                // milestone, when a line index justifies the change bookkeeping.
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: String::from("tessariql-language-server"),
                version: Some(String::from(env!("CARGO_PKG_VERSION"))),
            }),
            // UTF-16 is the LSP default and what the line index will speak
            // until a client negotiates something else.
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("tessariql-language-server {} up", env!("CARGO_PKG_VERSION")),
            )
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri.as_str()))]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let item = params.text_document;
        let id = DocumentId::from_uri(item.uri.as_str());
        self.documents.open(
            id,
            DocumentVersion::from_lsp(item.version),
            DocumentText::new(item.text),
        );
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri.as_str()))]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let id = DocumentId::from_uri(params.text_document.uri.as_str());
        let version = DocumentVersion::from_lsp(params.text_document.version);
        // Full sync: the client sends the whole new content. Taking the last
        // entry is defensive; the spec guarantees exactly one for FULL.
        let Some(change) = params.content_changes.into_iter().last() else {
            debug!("didChange with no content changes; ignoring");
            return;
        };
        if let Err(source) = self
            .documents
            .change(&id, version, DocumentText::new(change.text))
        {
            // A version regression means the editor and the server disagree on
            // ordering; the newer stored content is kept, the client hears why.
            warn!(%source, "didChange rejected; keeping the newer stored content");
            self.client
                .log_message(MessageType::WARNING, format!("change rejected: {source}"))
                .await;
        }
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri.as_str()))]
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let id = DocumentId::from_uri(params.text_document.uri.as_str());
        self.documents.close(&id);
    }
}

/// Builds the service/socket pair: the composition root's only wiring concern.
pub fn build(documents: Arc<Documents>) -> (LspService<Backend>, tower_lsp_server::ClientSocket) {
    LspService::build(|client| Backend { client, documents }).finish()
}
