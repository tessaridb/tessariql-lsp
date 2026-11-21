//! Hover: the word under the cursor explains itself. The word is scanned as
//! text — like completion, hover must work on a statement the tree cannot
//! fully place yet.

use tessariql_lsp_constants::docs::hover_doc;

/// Characters that form a hoverable word: the completion set plus `-` so
/// `ORDER BY` and `DEFINE TABLE` hover as phrases.
fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'$' | b'-')
}

/// The word under or immediately before `byte_offset`: expands both ways
/// when the cursor sits on the word, left only when it trails it. Single
/// words look up verbatim; up to two words before the cursor join into
/// phrase candidates (`ORDER BY`, `DEFINE TABLE`) — the longest phrase the
/// docs table knows wins.
#[must_use]
pub fn hover(source: &str, byte_offset: usize) -> Option<&'static str> {
    let bytes = source.as_bytes();
    let len = bytes.len();
    if len == 0 {
        return None;
    }
    let offset = byte_offset.min(len);
    let mut start = offset;
    while start > 0 && is_word_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    while end < len && is_word_byte(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    let word = &source[start..end];
    if let Some(text) = hover_doc(word) {
        return Some(text);
    }
    // Phrase candidates: the words immediately before the cursor's word.
    let before = source[..start].trim_end();
    let second_start = before
        .as_bytes()
        .iter()
        .rposition(|byte| !is_word_byte(*byte))
        .map_or(0, |index| index + 1);
    let second = &before[second_start..];
    if !second.is_empty() {
        let phrase = format!("{second} {word}");
        if let Some((_, text)) = tessariql_lsp_constants::docs::HOVER_DOCS
            .iter()
            .find(|(key, _)| *key == phrase)
        {
            return Some(text);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::hover;

    #[test]
    fn keyword_under_cursor_explains_itself() {
        // Cursor on the `L` of SELECT (offset 2).
        let text = hover("SELECT * FROM people;", 2);
        assert!(text.is_some_and(|text| text.starts_with("Reads records")));
    }

    #[test]
    fn multi_word_phrase_hovers_as_a_whole() {
        // Cursor after "ORDER BY" in a statement that stops there.
        let text = hover("SELECT * FROM people ORDER BY", 29);
        assert!(text.is_some_and(|text| text.contains("FUSE")));
    }

    #[test]
    fn function_name_hovers_with_family_text() {
        let text = hover("SELECT string::upper(name) FROM people;", 12);
        assert!(text.is_some_and(|text| text.contains("String function")));
    }

    #[test]
    fn plain_identifier_has_no_hover() {
        assert!(hover("SELECT name FROM people;", 10).is_none());
    }
}
