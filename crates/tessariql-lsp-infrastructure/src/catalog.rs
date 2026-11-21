//! The catalog cache: what exists on the node that an editor should offer
//! while typing — tables for the table positions, fields once a table is
//! known.
//!
//! The catalog is read with `INFO FOR` over the configured tenancy and is a
//! snapshot: it is fetched at connect and on demand, not watched. A shape
//! this build does not recognise reads as empty (a newer node), never as an
//! error the editor has to see.

use std::collections::BTreeMap;

use tessaridb_client::{Answer, Client, Value};
use tokio::net::TcpStream;

use crate::config::ConnectionConfig;
use crate::error::InfraResult;

/// Object keys the database-info answer may hold its tables under, in the
/// order this build tries them.
pub const TABLE_KEYS: &[&str] = &["tables", "table"];
/// Object keys a table-info answer may hold its fields under.
pub const FIELD_KEYS: &[&str] = &["fields", "field"];

/// A point-in-time view of the configured tenancy's catalog.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Catalog {
    /// Table names, sorted — what the table positions complete.
    pub tables: Vec<String>,
    /// Field names per table — what a WHERE clause completes.
    pub fields: BTreeMap<String, Vec<String>>,
}

impl Catalog {
    /// Fetches the catalog over the connection: `INFO FOR DATABASE` for the
    /// table list, then one `INFO FOR TABLE` per table for its fields, all
    /// in two round trips.
    pub async fn fetch(
        client: &mut Client<TcpStream>,
        config: &ConnectionConfig,
    ) -> InfraResult<Self> {
        let mut catalog = Self::default();
        let answers = client
            .run(&info_script(config, None), config.credentials())
            .await?;
        let Some(database_info) = answers.last() else {
            return Ok(catalog);
        };
        catalog.tables = object_keys(database_info, TABLE_KEYS);
        if catalog.tables.is_empty() {
            return Ok(catalog);
        }
        // Names become script text; only names that lex as one identifier go
        // in — a strange table name reads as no fields, never as a rewrite.
        let tables: Vec<String> = catalog
            .tables
            .iter()
            .filter(|table| is_catalog_name(table))
            .cloned()
            .collect();
        let answers = client
            .run(&info_script(config, Some(&tables)), config.credentials())
            .await?;
        for (table, answer) in tables.iter().zip(answers.iter()) {
            catalog
                .fields
                .insert(table.clone(), object_keys(answer, FIELD_KEYS));
        }
        Ok(catalog)
    }
}

/// The USE prefix (when a tenancy is configured) plus the INFO statements:
/// the database info, or one table info per named table. Names are grammar,
/// so they are validated as names — never bound, never quoted here.
fn info_script(config: &ConnectionConfig, tables: Option<&[String]>) -> String {
    let mut script = String::new();
    if let Some(namespace) = config
        .namespace
        .as_ref()
        .filter(|name| is_catalog_name(name))
    {
        script.push_str(&format!("USE NAMESPACE {namespace}; "));
    }
    if let Some(database) = config
        .database
        .as_ref()
        .filter(|name| is_catalog_name(name))
    {
        script.push_str(&format!("USE DATABASE {database}; "));
    }
    match tables {
        None => script.push_str("INFO FOR DATABASE;"),
        Some(tables) => {
            for table in tables {
                script.push_str(&format!("INFO FOR TABLE {table}; "));
            }
        }
    }
    script
}

/// Validates a catalog name before it becomes script text (a name is grammar;
/// M5 — names are checked, never bound). Returns None when it would not lex
/// as one identifier.
#[must_use]
pub fn is_catalog_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_alphabetic() || first == '_' => {}
        _ => return false,
    }
    name.chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_')
}

/// The names in the first candidate container found under `keys` in the
/// answer's value. 0.50 answers these as arrays — tables as strings, fields
/// as objects carrying a `name` — and older shapes were objects keyed by
/// name; all three read here, anything else reads as empty (a newer node),
/// never as an error the editor has to see.
fn object_keys(answer: &Answer, keys: &[&str]) -> Vec<String> {
    let Answer::Value { value, .. } = answer else {
        return Vec::new();
    };
    let Value::Object(map) = value else {
        return Vec::new();
    };
    for key in keys {
        match map.get(*key) {
            Some(Value::Object(entries)) => return entries.keys().cloned().collect(),
            Some(Value::Array(items)) => {
                return items
                    .iter()
                    .filter_map(|item| match item {
                        Value::String(name) => Some(name.clone()),
                        Value::Object(fields) => match fields.get("name") {
                            Some(Value::String(name)) => Some(name.clone()),
                            _ => None,
                        },
                        _ => None,
                    })
                    .collect();
            }
            _ => {}
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::{FIELD_KEYS, TABLE_KEYS, is_catalog_name, object_keys};
    use tessaridb_client::{Answer, Value};

    #[test]
    fn catalog_names_are_checked_before_they_become_script() {
        assert!(is_catalog_name("people"));
        assert!(is_catalog_name("_private2"));
        assert!(!is_catalog_name("2bad"));
        assert!(!is_catalog_name("bad-name"));
        assert!(!is_catalog_name("bad name"));
        assert!(!is_catalog_name("bad; DROP TABLE x"));
    }

    #[test]
    fn object_keys_reads_the_first_matching_container() {
        let mut tables = std::collections::BTreeMap::new();
        tables.insert(String::from("users"), Value::Null);
        tables.insert(String::from("orders"), Value::Null);
        let mut map = std::collections::BTreeMap::new();
        map.insert(String::from("tables"), Value::Object(tables));
        let answer = Answer::Value {
            value: Value::Object(map),
            names: Default::default(),
        };
        assert_eq!(object_keys(&answer, TABLE_KEYS), vec!["orders", "users"]);
        assert!(object_keys(&answer, FIELD_KEYS).is_empty());
    }

    #[test]
    fn a_non_object_answer_reads_as_empty_catalog() {
        assert!(object_keys(&Answer::Done, TABLE_KEYS).is_empty());
        assert!(object_keys(&Answer::Removed(3), TABLE_KEYS).is_empty());
    }
}
