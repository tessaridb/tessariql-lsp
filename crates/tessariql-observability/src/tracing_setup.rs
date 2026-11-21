use tessariql_constants::LSP_SPEC_VERSION;
use tracing_subscriber::EnvFilter;

/// Installs the tracing subscriber. `TESSARIQL_LSP_LOG` selects the filter
/// (default `info`); `TESSARIQL_LSP_LOG_FORMAT=json` switches from the
/// human-readable format to JSON for log aggregation.
pub fn init() {
    let filter =
        EnvFilter::try_from_env("TESSARIQL_LSP_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    let json = std::env::var("TESSARIQL_LSP_LOG_FORMAT")
        .map(|value| value.eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    let registry = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    if json {
        registry.json().init();
    } else {
        registry.init();
    }
    tracing::info!(spec = LSP_SPEC_VERSION, "tessariql-lsp tracing up");
}
