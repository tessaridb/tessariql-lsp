//! Binary crate: the LSP adapter over the domain, plus the composition root.
//!
//! The adapter translates protocol calls into domain calls and back. Document
//! state lives in `tessariql-core`; this crate holds no business rules — when
//! the transport surface outgrows one file, it splits into its own crate
//! (recorded decision, see docs/DESIGN.md §Structure).

mod adapter;

use std::process::ExitCode;

use tessariql_core::Documents;

#[tokio::main]
async fn main() -> ExitCode {
    tessariql_observability::init();

    let documents = Documents::new().shared();
    let (service, socket) = adapter::build(documents);
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
