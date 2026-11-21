//! Syntax diagnostics over the error-tolerant tree.
//!
//! The grammar never fails to produce a tree; what it cannot place becomes
//! `ERROR` nodes, and what a statement left out becomes `MISSING` nodes.
//! Both are diagnostics here — an unparseable corner is not silent.

use tree_sitter::{Node, Parser, Point};

/// A position in a document: zero-based row, column counted in bytes
/// (tree-sitter's native unit; callers convert per line for LSP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytePoint {
    pub row: usize,
    pub column: usize,
}

/// How bad the finding is. The grammar only produces errors today; the enum
/// leaves room for warnings (deprecated clauses) without an API change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
}

/// One syntax finding over a byte range of the source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub start: BytePoint,
    pub end: BytePoint,
    pub severity: Severity,
    pub message: String,
}

impl Diagnostic {
    fn error(start: Point, end: Point, message: String) -> Self {
        Self {
            start: BytePoint {
                row: start.row,
                column: start.column,
            },
            end: BytePoint {
                row: end.row,
                column: end.column,
            },
            severity: Severity::Error,
            message,
        }
    }
}

/// Parses `source` and collects every ERROR and MISSING node as a diagnostic.
/// An empty parse (no tree) cannot happen — tree-sitter always answers — but
/// a defensive empty Vec is returned rather than a panic.
#[must_use]
pub fn analyze(source: &str) -> Vec<Diagnostic> {
    let mut parser = Parser::new();
    if parser.set_language(&crate::language()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };

    let mut diagnostics = Vec::new();
    collect_errors(tree.root_node(), &mut diagnostics);
    diagnostics
}

/// Depth-first walk; ERROR nodes are reported with their whole span, MISSING
/// nodes with the token they stand for. A MISSING node is zero-width at the
/// insertion point.
fn collect_errors(node: Node, diagnostics: &mut Vec<Diagnostic>) {
    if node.is_error() {
        diagnostics.push(Diagnostic::error(
            node.start_position(),
            node.end_position(),
            String::from("syntax error"),
        ));
        return;
    }
    if node.is_missing() {
        diagnostics.push(Diagnostic::error(
            node.start_position(),
            node.end_position(),
            format!("missing {}", node.kind()),
        ));
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_errors(child, diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::analyze;

    #[test]
    fn clean_statement_has_no_diagnostics() {
        assert!(analyze("SELECT * FROM people;").is_empty());
    }

    #[test]
    fn broken_statement_reports_an_error_span() {
        let diagnostics = analyze("SELEC * FROM people;");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "syntax error");
        assert!(diagnostics[0].start.row == 0);
    }

    #[test]
    fn truncated_statement_reports_an_error() {
        // The clause-repeat grammar folds an unplaceable trailing clause into
        // an ERROR node; the MISSING branch is the defensive path for shapes
        // a stricter production leaves incomplete.
        let diagnostics = analyze("SELECT * FROM;");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "syntax error");
    }
}
