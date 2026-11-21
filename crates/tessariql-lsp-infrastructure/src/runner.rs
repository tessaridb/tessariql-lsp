//! The node session: one signed-in connection, a statement runner for the
//! run-under-cursor command, and the catalog refresh.

use tessaridb_client::{Answer, Client};
use tokio::net::TcpStream;

use crate::catalog::Catalog;
use crate::config::ConnectionConfig;
use crate::error::InfraResult;

/// One connection to the node. The client is a single stream — not
/// shareable across tasks — so the session serialises callers with a mutex
/// at the adapter level, and every statement runs in its own call.
#[derive(Debug)]
pub struct NodeSession {
    client: Client<TcpStream>,
    config: ConnectionConfig,
}

impl NodeSession {
    /// Connects and signs in on the first statement, per the client's design.
    pub async fn connect(config: ConnectionConfig) -> InfraResult<Self> {
        let client = Client::connect(config.address.as_str()).await?;
        tracing::info!(node = %config.address, "connected for catalog and run features");
        Ok(Self { client, config })
    }

    /// The configured tenancy's catalog snapshot.
    pub async fn catalog(&mut self) -> InfraResult<Catalog> {
        Catalog::fetch(&mut self.client, &self.config).await
    }

    /// Runs one statement and renders its answers as the text an editor
    /// shows for the run command. A refusal travels as the node's own words
    /// in the error; the caller renders that too.
    pub async fn run_statement(&mut self, statement: &str) -> InfraResult<String> {
        let answers = self
            .client
            .run(statement, self.config.credentials())
            .await?;
        Ok(render_answers(&answers))
    }
}

/// Answers as display text, one line per statement answer, mirroring how the
/// CLI speaks: writes say ok, reads say how many via which path, values use
/// their debug form until a rendering spec for the language exists.
#[must_use]
pub fn render_answers(answers: &[Answer]) -> String {
    let mut lines = Vec::with_capacity(answers.len());
    for answer in answers {
        lines.push(match answer {
            Answer::Done => String::from("ok"),
            Answer::Records { records, path, .. } => {
                let mut line = format!("{} record(s), via {path}", records.len());
                for (id, value) in records {
                    line.push_str(&format!("\n  {id}: {value:?}"));
                }
                line
            }
            Answer::Value { value, .. } => format!("{value:?}"),
            Answer::Keys(keys) => keys.join("\n"),
            Answer::Removed(count) => format!("{count} removed"),
            Answer::Unknown => String::from("(answer this build does not know)"),
            // The enum is non-exhaustive on purpose; a newer node may add a
            // variant, and the honest rendering says so rather than dropping it.
            _ => String::from("(answer this build does not know)"),
        });
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::render_answers;
    use tessaridb_client::Answer;

    #[test]
    fn done_renders_ok() {
        assert_eq!(render_answers(&[Answer::Done]), "ok");
    }

    #[test]
    fn unknown_answers_say_so() {
        assert_eq!(
            render_answers(&[Answer::Unknown]),
            "(answer this build does not know)"
        );
    }

    #[test]
    fn multiple_answers_join_with_newlines() {
        assert_eq!(
            render_answers(&[Answer::Done, Answer::Removed(2)]),
            "ok\n2 removed"
        );
    }
}
