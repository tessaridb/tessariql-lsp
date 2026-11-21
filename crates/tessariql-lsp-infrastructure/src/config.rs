//! The connection configuration — the feature flag. `TESSARIQL_LSP_NODE`
//! set to a `host:port` turns the catalog and run features on; without it
//! nothing in this crate is constructed.
//!
//! The password arrives from the environment only, never an argument, so
//! the process table never publishes it. A user without a password (or the
//! reverse) is a half configuration and refuses rather than guessing.

use tessariql_lsp_constants::environment::{
    DATABASE_ENV, NAMESPACE_ENV, NODE_ENV, PASSWORD_ENV, USER_ENV,
};

/// How to reach the node, and as whom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfig {
    /// The node's wire address, `host:port`.
    pub address: String,
    /// Signed in as this user; the password travels per call, never stored.
    pub user: Option<String>,
    /// The node's answer to the password challenge. None for an open store.
    pub password: Option<String>,
    /// Tenancy the catalog reads; absent reads whatever the connection holds.
    pub namespace: Option<String>,
    pub database: Option<String>,
}

impl ConnectionConfig {
    /// Reads the environment. `None` means the feature is off (no address),
    /// or the configuration is half-written (user without password or the
    /// reverse) — the caller logs and runs without catalog features.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Self::from_vars(
            std::env::var(NODE_ENV).ok(),
            std::env::var(USER_ENV).ok(),
            std::env::var(PASSWORD_ENV).ok(),
            std::env::var(NAMESPACE_ENV).ok(),
            std::env::var(DATABASE_ENV).ok(),
        )
    }

    /// The decision body, separated from the environment so it tests without
    /// touching process-wide state.
    #[must_use]
    pub fn from_vars(
        node: Option<String>,
        user: Option<String>,
        password: Option<String>,
        namespace: Option<String>,
        database: Option<String>,
    ) -> Option<Self> {
        let address = node.filter(|value| !value.trim().is_empty())?;
        let (user, password) = match (
            user.filter(|v| !v.is_empty()),
            password.filter(|v| !v.is_empty()),
        ) {
            (Some(user), Some(password)) => (Some(user), Some(password)),
            (None, None) => (None, None),
            // Half a credential is not a credential: refuse rather than sign
            // in anonymously against a store that expected a user.
            _ => return None,
        };
        Some(Self {
            address,
            user,
            password,
            namespace: namespace.filter(|v| !v.is_empty()),
            database: database.filter(|v| !v.is_empty()),
        })
    }

    /// Credentials as the client wants them: `Some((user, password))` or None.
    #[must_use]
    pub fn credentials(&self) -> Option<(&str, &str)> {
        match (&self.user, &self.password) {
            (Some(user), Some(password)) => Some((user.as_str(), password.as_str())),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionConfig;

    #[test]
    fn no_node_means_feature_off() {
        assert!(ConnectionConfig::from_vars(None, None, None, None, None).is_none());
        assert!(ConnectionConfig::from_vars(Some(String::new()), None, None, None, None).is_none());
    }

    #[test]
    fn node_alone_is_a_valid_open_store() {
        let config = ConnectionConfig::from_vars(
            Some(String::from("127.0.0.1:47920")),
            None,
            None,
            None,
            None,
        )
        .expect("an address alone connects to an open store");
        assert!(config.credentials().is_none());
    }

    #[test]
    fn half_a_credential_refuses() {
        assert!(
            ConnectionConfig::from_vars(
                Some(String::from("127.0.0.1:47920")),
                Some(String::from("root")),
                None,
                None,
                None,
            )
            .is_none(),
            "a user without a password is not a configuration"
        );
    }

    #[test]
    fn full_credentials_round_trip() {
        let config = ConnectionConfig::from_vars(
            Some(String::from("127.0.0.1:47920")),
            Some(String::from("root")),
            Some(String::from("pw")),
            Some(String::from("prod")),
            Some(String::from("orders")),
        )
        .expect("complete configuration parses");
        assert_eq!(config.credentials(), Some(("root", "pw")));
        assert_eq!(config.namespace.as_deref(), Some("prod"));
    }
}
