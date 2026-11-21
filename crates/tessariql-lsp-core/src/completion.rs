//! Completion: filter the static TessariQL vocabulary by the word prefix
//! under the cursor. Deliberately simple — the prefix is scanned as text, so
//! completion works in a half-typed statement the tree cannot place yet,
//! which is exactly where an editor asks for it.

use tessariql_lsp_constants::language::{FUNCTIONS, KEYWORDS, STATEMENT_STARTERS};

/// What a completion entry is, so the adapter can map it to an LSP kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Keyword,
    Statement,
    Function,
}

/// One offer: the label is a static catalog string, borrowed for the answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Completion {
    pub label: &'static str,
    pub kind: CompletionKind,
}

/// Characters that form a completable prefix: identifiers, the namespace
/// separator (so `vector::co` completes `vector::cosine`), and `$` so a
/// parameter name can be completed after its sigil.
fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'$')
}

/// The word prefix ending at `byte_offset`, empty when the cursor sits after
/// whitespace or punctuation (the common "show me everything" case).
fn prefix_at(source: &str, byte_offset: usize) -> &str {
    let bytes = source.as_bytes();
    let end = byte_offset.min(bytes.len());
    let mut start = end;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    &source[start..end]
}

/// Every catalog entry the prefix matches, namespace-aware: a prefix
/// containing `::` only searches the function catalog. Results are ordered
/// keywords, statement starters, functions; catalog duplicates collapse.
#[must_use]
pub fn complete(source: &str, byte_offset: usize) -> Vec<Completion> {
    let prefix = prefix_at(source, byte_offset);
    let mut out: Vec<Completion> = Vec::new();
    if prefix.contains("::") || prefix.starts_with('$') {
        push_matches(&mut out, FUNCTIONS, CompletionKind::Function, prefix);
        return out;
    }
    push_matches(&mut out, KEYWORDS, CompletionKind::Keyword, prefix);
    push_matches(
        &mut out,
        STATEMENT_STARTERS,
        CompletionKind::Statement,
        prefix,
    );
    push_matches(&mut out, FUNCTIONS, CompletionKind::Function, prefix);
    out
}

fn push_matches(
    out: &mut Vec<Completion>,
    catalog: &[&'static str],
    kind: CompletionKind,
    prefix: &str,
) {
    for label in catalog {
        if label.starts_with(prefix) && !out.iter().any(|entry| entry.label == *label) {
            out.push(Completion { label, kind });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CompletionKind, complete};

    #[test]
    fn prefix_filters_keywords() {
        let source = "SEL";
        let found = complete(source, source.len());
        assert!(
            found
                .iter()
                .any(|entry| entry.label == "SELECT" && entry.kind == CompletionKind::Keyword)
        );
        assert!(found.iter().all(|entry| entry.label.starts_with("SEL")));
    }

    #[test]
    fn namespace_prefix_completes_functions_only() {
        let source = "SELECT vector::co";
        let found = complete(source, source.len());
        assert!(
            found
                .iter()
                .any(|entry| entry.label == "vector::cosine"
                    && entry.kind == CompletionKind::Function)
        );
        assert!(
            found
                .iter()
                .all(|entry| entry.kind == CompletionKind::Function)
        );
    }

    #[test]
    fn empty_prefix_lists_everything() {
        let found = complete("", 0);
        assert!(found.len() > 100);
    }

    #[test]
    fn mid_document_offset_uses_the_word_before_the_cursor() {
        // Cursor right after `SEL`, with the rest of the statement already
        // sitting behind it — completion must not read past the cursor.
        let source = "SEL FROM people;";
        let found = complete(source, 3);
        assert!(found.iter().any(|entry| entry.label == "SELECT"));
        assert!(found.iter().all(|entry| entry.label.starts_with("SEL")));
    }
}
