//! Infrastructure for the TessariQL language server: the connection to a
//! running node — catalog cache for completion and a statement runner for
//! the run-under-cursor command.
//!
//! Everything here sits behind the `TESSARIQL_LSP_NODE` feature flag: with
//! no address configured, none of it is constructed and the server behaves
//! exactly as it did before this crate existed.

pub mod catalog;
pub mod config;
pub mod error;
pub mod runner;

pub use catalog::{Catalog, TABLE_KEYS};
pub use config::ConnectionConfig;
pub use error::{InfraError, InfraResult};
pub use runner::{NodeSession, render_answers};
