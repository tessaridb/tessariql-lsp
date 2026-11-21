//! The LSP adapter: protocol calls in, domain calls, protocol answers out.
//!
//! Everything here is translation. State transitions live in
//! [`tessariql_lsp_core`]; shared-state discipline: every domain call is a short
//! synchronous section, the result is collected, and no guard is held across
//! an `.await` in this file.

use std::sync::Arc;

use tessariql_lsp_core::{CompletionKind, Documents, complete, hover};
use tessariql_lsp_types::{DocumentId, DocumentText, DocumentVersion};
use tower_lsp_server::jsonrpc::Result as LspResult;
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    Diagnostic as LspDiagnostic, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover, HoverContents, HoverParams,
    InitializeParams, InitializeResult, InitializedParams, MarkedString, MessageType, Position,
    Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use tower_lsp_server::{Client, LanguageServer, LspService};
use tracing::{debug, instrument, warn};
use tree_sitter_tessariql::{Diagnostic as CoreDiagnostic, analyze};

/// Handler state: the editor-facing client channel and the open documents.
#[derive(Debug)]
pub struct Backend {
    client: Client,
    documents: Arc<Documents>,
}

impl Backend {
    /// Parses the stored document and pushes the findings. The document is
    /// cloned out of the registry first, so no DashMap guard crosses the
    /// analysis or the `.await`; a document that raced a close is skipped.
    async fn refresh_diagnostics(&self, id: &DocumentId, uri: Uri) {
        let Some(document) = self.documents.get(id) else {
            return;
        };
        let source = document.text().as_str();
        let diagnostics = analyze(source)
            .into_iter()
            .map(|finding| to_lsp_diagnostic(source, &finding))
            .collect();
        self.client
            .publish_diagnostics(uri, diagnostics, Some(document.version().as_i32()))
            .await;
    }
}

/// One domain finding as the protocol shape: rows pass through, byte columns
/// become character columns for the line (LSP speaks UTF-16 units in spirit;
/// counting chars is exact for the ASCII surface of the language and honest
/// for anything else until an incremental line index lands).
fn to_lsp_diagnostic(source: &str, finding: &CoreDiagnostic) -> LspDiagnostic {
    LspDiagnostic {
        range: Range {
            start: to_lsp_position(source, finding.start.row, finding.start.column),
            end: to_lsp_position(source, finding.end.row, finding.end.column),
        },
        severity: Some(DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some(String::from("tessariql")),
        message: finding.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}

fn to_lsp_position(source: &str, row: usize, byte_column: usize) -> Position {
    let line = source.lines().nth(row).unwrap_or_default();
    let upto = line.len().min(byte_column);
    // A document with more lines or columns than u32::MAX cannot be addressed
    // in LSP positions at all; saturating keeps the server alive for it.
    let character = u32::try_from(line[..upto].chars().count()).unwrap_or(u32::MAX);
    Position {
        line: u32::try_from(row).unwrap_or(u32::MAX),
        character,
    }
}

/// The byte offset in `source` of an LSP position: the line's start plus the
/// byte index of the `character`-th char (clamped at the line end).
fn position_to_byte_offset(source: &str, position: &Position) -> usize {
    let Ok(row) = usize::try_from(position.line) else {
        return source.len();
    };
    let Some(line) = source.lines().nth(row) else {
        return source.len();
    };
    let Ok(character) = usize::try_from(position.character) else {
        return line.len();
    };
    line.char_indices()
        .nth(character)
        .map_or(line.len(), |(byte_index, _)| byte_index)
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
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![String::from(":"), String::from("$")]),
                    ..CompletionOptions::default()
                }),
                hover_provider: Some(tower_lsp_server::ls_types::HoverProviderCapability::Simple(
                    true,
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

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let id = DocumentId::from_uri(uri.as_str());
        let Some(document) = self.documents.get(&id) else {
            return Ok(None);
        };
        let source = document.text().as_str();
        let offset = position_to_byte_offset(source, &params.text_document_position.position);
        let items = complete(source, offset)
            .into_iter()
            .map(|entry| CompletionItem {
                label: String::from(entry.label),
                kind: Some(match entry.kind {
                    CompletionKind::Function => CompletionItemKind::FUNCTION,
                    CompletionKind::Keyword | CompletionKind::Statement => {
                        CompletionItemKind::KEYWORD
                    }
                }),
                ..CompletionItem::default()
            })
            .collect();
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let id = DocumentId::from_uri(uri.as_str());
        let Some(document) = self.documents.get(&id) else {
            return Ok(None);
        };
        let source = document.text().as_str();
        let offset =
            position_to_byte_offset(source, &params.text_document_position_params.position);
        let Some(text) = hover(source, offset) else {
            return Ok(None);
        };
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(String::from(text))),
            range: None,
        }))
    }

    #[instrument(skip(self, params), fields(uri = %params.text_document.uri.as_str()))]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let item = params.text_document;
        let id = DocumentId::from_uri(item.uri.as_str());
        self.documents.open(
            id.clone(),
            DocumentVersion::from_lsp(item.version),
            DocumentText::new(item.text),
        );
        self.refresh_diagnostics(&id, item.uri).await;
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
        match self
            .documents
            .change(&id, version, DocumentText::new(change.text))
        {
            Ok(()) => {
                self.refresh_diagnostics(&id, params.text_document.uri)
                    .await
            }
            Err(source) => {
                // A version regression means the editor and the server disagree on
                // ordering; the newer stored content is kept, the client hears why.
                warn!(%source, "didChange rejected; keeping the newer stored content");
                self.client
                    .log_message(MessageType::WARNING, format!("change rejected: {source}"))
                    .await;
            }
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
