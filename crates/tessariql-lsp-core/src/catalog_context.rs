//! Catalog completion context: what kind of catalog object the cursor is in
//! a position to name. Textual and deliberately small — tables after the
//! table-naming keywords, fields of the statement's own table inside its
//! WHERE — enough for the editor to ask the catalog the right question.

/// What the position under the cursor can complete from the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogContext {
    /// After FROM / ON / TABLE / INTO / UPDATE / JOIN: a table name.
    Table,
    /// Inside a WHERE whose statement names this table: a field of it.
    Field { table: String },
}

/// The keywords whose following word names a table.
const TABLE_KEYWORDS: &[&str] = &["FROM", "ON", "TABLE", "INTO", "UPDATE", "JOIN"];

/// Decides the catalog question for `byte_offset` in `source`. Looks only at
/// the statement segment the cursor sits in (statements are `;`-terminated).
#[must_use]
pub fn catalog_context(source: &str, byte_offset: usize) -> Option<CatalogContext> {
    let bytes = source.as_bytes();
    let end = byte_offset.min(bytes.len());
    // The statement segment containing the cursor: back to the last `;`.
    let segment_start = source[..end].rfind(';').map_or(0, |index| index + 1);
    // Whether the cursor stands inside or right after a word (no whitespace
    // before it) — decided on the raw text, before any trim.
    let at_word = end > segment_start && source.as_bytes()[end - 1].is_ascii_alphanumeric()
        || (end > segment_start && source.as_bytes()[end - 1] == b'_');
    let before = source[segment_start..end].trim_end();
    let mut tokens: Vec<&str> = before.split_whitespace().collect();
    // A word the cursor is still typing is not context.
    if at_word {
        tokens.pop();
    }
    let last = *tokens.last()?;
    if TABLE_KEYWORDS.contains(&last.to_ascii_uppercase().as_str()) {
        return Some(CatalogContext::Table);
    }
    // Field position: a WHERE opened before the cursor, and the statement
    // names one table after FROM/JOIN/UPDATE.
    let where_seen = tokens
        .iter()
        .any(|token| token.eq_ignore_ascii_case("WHERE"));
    if where_seen {
        for window in tokens.windows(2) {
            if TABLE_KEYWORDS.contains(&window[0].to_ascii_uppercase().as_str())
                && is_identifier(window[1])
            {
                return Some(CatalogContext::Field {
                    table: String::from(window[1]),
                });
            }
        }
    }
    None
}

/// A token that is clearly a prefix being typed: bare word characters, no
/// quotes or punctuation.
fn is_partial(token: &str) -> bool {
    !token.is_empty()
        && token
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_')
        && token
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
}

/// The same shape, used where the token must already be a whole name.
fn is_identifier(token: &str) -> bool {
    is_partial(token)
}

#[cfg(test)]
mod tests {
    use super::{CatalogContext, catalog_context};

    #[test]
    fn after_from_asks_for_tables() {
        let source = "SELECT * FROM ";
        assert_eq!(
            catalog_context(source, source.len()),
            Some(CatalogContext::Table)
        );
    }

    #[test]
    fn mid_prefix_after_from_still_tables() {
        let source = "SELECT * FROM pe";
        assert_eq!(
            catalog_context(source, source.len()),
            Some(CatalogContext::Table)
        );
    }

    #[test]
    fn inside_where_asks_for_fields_of_the_statements_table() {
        let source = "SELECT * FROM people WHERE ";
        assert_eq!(
            catalog_context(source, source.len()),
            Some(CatalogContext::Field {
                table: String::from("people")
            })
        );
    }

    #[test]
    fn earlier_statements_do_not_leak_in() {
        let source = "USE NAMESPACE prod;\nUSE DATABASE orders;\nSELECT * FROM ";
        assert_eq!(
            catalog_context(source, source.len()),
            Some(CatalogContext::Table)
        );
    }

    #[test]
    fn select_list_position_has_no_catalog_context() {
        let source = "SELECT na";
        assert_eq!(catalog_context(source, source.len()), None);
    }
}
