//! Rust binding for the TessariQL tree-sitter grammar.
//!
//! The grammar is error-tolerant by construction (tree-sitter always
//! produces a tree) and is validated against the engine statement corpus in
//! `../../corpus/statements` — every statement there parses with no `ERROR`
//! and no `MISSING` node, and mutated inputs never panic the parser.

pub mod diagnostics;

pub use diagnostics::{BytePoint, Diagnostic, Severity, analyze};

unsafe extern "C" {
    #[link_name = "tree_sitter_tessariql"]
    fn ts_language() -> tree_sitter::Language;
}

/// The TessariQL language.
#[must_use]
pub fn language() -> tree_sitter::Language {
    unsafe { ts_language() }
}

#[cfg(test)]
mod tests {
    use super::language;

    const CORPUS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/statements");

    /// Every engine-validated corpus statement parses without ERROR or
    /// MISSING nodes. The corpus is the grammar's conformance suite.
    #[test]
    fn corpus_statements_parse_clean() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language())
            .expect("language loads into the parser");
        let dir = std::path::Path::new(CORPUS_DIR);
        let mut files: Vec<_> = std::fs::read_dir(dir)
            .expect("corpus directory is readable")
            .map(|entry| entry.expect("corpus entry is readable").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "tessariql"))
            .collect();
        files.sort();
        assert!(
            files.len() >= 60,
            "corpus holds the SC3 minimum of 60 statements"
        );
        for path in &files {
            let source = std::fs::read_to_string(path).expect("corpus statement is readable");
            let tree = parser.parse(&source, None).expect("parse returns a tree");
            let root = tree.root_node();
            assert!(
                !root.has_error(),
                "{} parses without ERROR nodes\n{}",
                path.display(),
                root.to_sexp()
            );
            let mut missing = false;
            let mut cursor = root.walk();
            for node in root.children(&mut cursor) {
                if node.is_missing() {
                    missing = true;
                }
            }
            assert!(!missing, "{} has no MISSING nodes", path.display());
        }
    }

    /// Mutation smoke: deleting, duplicating, swapping and truncating bytes
    /// must never panic the parser and must always return a tree. Seeded for
    /// reproducibility.
    #[test]
    fn mutations_never_panic() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&language())
            .expect("language loads into the parser");
        let dir = std::path::Path::new(CORPUS_DIR);
        let mut sources: Vec<String> = std::fs::read_dir(dir)
            .expect("corpus directory is readable")
            .map(|entry| entry.expect("corpus entry is readable").path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "tessariql"))
            .map(|path| std::fs::read_to_string(path).expect("corpus statement is readable"))
            .collect();
        sources.sort();

        // xorshift64* — deterministic, no external crates.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state.wrapping_mul(0x2545_F491_4F6C_DD1D)
        };

        let mut total = 0usize;
        for source in &sources {
            let bytes = source.as_bytes();
            for _ in 0..30 {
                if bytes.is_empty() {
                    continue;
                }
                let mutated = match next() % 4 {
                    0 => {
                        // delete a byte
                        let at = (next() as usize) % bytes.len();
                        let mut v = bytes.to_vec();
                        v.remove(at);
                        v
                    }
                    1 => {
                        // duplicate a byte
                        let at = (next() as usize) % bytes.len();
                        let mut v = bytes.to_vec();
                        v.insert(at, bytes[at]);
                        v
                    }
                    2 => {
                        // swap two bytes
                        let a = (next() as usize) % bytes.len();
                        let b = (next() as usize) % bytes.len();
                        let mut v = bytes.to_vec();
                        v.swap(a, b);
                        v
                    }
                    _ => {
                        // truncate
                        let len = (next() as usize) % bytes.len();
                        bytes[..len].to_vec()
                    }
                };
                total += 1;
                let text = String::from_utf8_lossy(&mutated);
                let tree = parser.parse(text.as_bytes(), None);
                assert!(
                    tree.is_some(),
                    "parser always returns a tree (mutation {total})"
                );
            }
        }
        assert!(
            total >= 5_000,
            "mutation smoke covers at least 5000 inputs, ran {total}"
        );
    }
}
