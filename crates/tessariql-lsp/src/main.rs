//! Binary crate: the LSP adapter over the domain, plus the composition root.
//!
//! The adapter translates protocol calls into domain calls and back. Document
//! state lives in `tessariql-lsp-core`; this crate holds no business rules — when
//! the transport surface outgrows one file, it splits into its own crate
//! (recorded decision, see docs/DESIGN.md §Structure).

mod adapter;

use std::process::ExitCode;
use std::sync::Arc;

use tessariql_lsp_core::Documents;
use tessariql_lsp_infrastructure::{ConnectionConfig, NodeSession};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> ExitCode {
    tessariql_lsp_observability::init();

    // The catalog/run features exist only when a node is configured; a
    // unreachable or refusing node degrades to the SG4 server, never a
    // failed start — the editor still gets diagnostics and completion.
    let (node, catalog) = match ConnectionConfig::from_env() {
        Some(config) => match NodeSession::connect(config.clone()).await {
            Ok(mut session) => match session.catalog().await {
                Ok(catalog) => {
                    tracing::info!(
                        tables = catalog.tables.len(),
                        "catalog loaded; catalog completion and run enabled"
                    );
                    (Some(Arc::new(Mutex::new(session))), Some(Arc::new(catalog)))
                }
                Err(source) => {
                    tracing::warn!(%source, "catalog fetch failed; running without catalog features");
                    (None, None)
                }
            },
            Err(source) => {
                tracing::warn!(%source, "node connect failed; running without catalog features");
                (None, None)
            }
        },
        None => (None, None),
    };

    let documents = Documents::new().shared();
    let (service, socket) = adapter::build(documents, node, catalog);
    let server = tower_lsp_server::Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service);

    tokio::select! {
        biased; // shutdown first: a signal outranks protocol traffic
        () = shutdown_signal() => {
            tracing::info!("signal received; stopping");
            ExitCode::SUCCESS
        }
        () = server => ExitCode::SUCCESS,
    }
}

/// SIGINT/SIGTERM. There is no persistent state to drain — documents live in
/// memory and die with the process — so graceful here means prompt.
#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = signal(SignalKind::interrupt()).expect("SIGINT handler installs");
    let mut sigterm = signal(SignalKind::terminate()).expect("SIGTERM handler installs");
    tokio::select! {
        _ = sigint.recv() => {}
        _ = sigterm.recv() => {}
    }
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.expect("ctrl-c handler");
}
