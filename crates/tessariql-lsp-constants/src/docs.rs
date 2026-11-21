//! Hover text for the editor's explain-under-cursor: one line per statement
//! or clause, written from the 0.50.0 docs pages (query-language/*.md); the
//! trailing link is where the full page lives. Function entries are family
//! text generated from the catalog in `language.rs`.

use crate::language::FUNCTIONS;

/// One-line hover text for statements and the clauses that carry them.
/// Written from the 0.50.0 docs pages (query-language/*.md); the trailing
/// link is where the full page lives.
pub const HOVER_DOCS: &[(&str, &str)] = &[
    (
        "SELECT",
        "Reads records from a table, view, or traversal — projections, conditions, grouping, ordering, paging. See https://docs.tessaridb.com/query-language/documents",
    ),
    (
        "CREATE",
        "Writes a new record; `=` replaces the whole record. See https://docs.tessaridb.com/query-language/documents",
    ),
    (
        "UPDATE",
        "Changes records: SET merges fields, `=` replaces, MERGE patches; WHERE states the reach. See https://docs.tessaridb.com/query-language/writing-safely",
    ),
    (
        "UPSERT",
        "Creates the record when absent, replaces it when there. See https://docs.tessaridb.com/query-language/writing-safely",
    ),
    (
        "DELETE",
        "Removes records; a set delete states its condition, never omits it. See https://docs.tessaridb.com/query-language/writing-safely",
    ),
    (
        "INSERT",
        "Inserts rows from a VALUES list. See https://docs.tessaridb.com/query-language/writing-safely",
    ),
    (
        "RELATE",
        "Writes an edge between two records in a graph. See https://docs.tessaridb.com/query-language/graphs",
    ),
    (
        "DEFINE",
        "Declares catalog objects — tables, fields, indexes, users, engines. See https://docs.tessaridb.com/query-language/definitions",
    ),
    (
        "DEFINE TABLE",
        "Declares a records table: columns, identity, storage scheme, split points. See https://docs.tessaridb.com/query-language/definitions",
    ),
    (
        "DEFINE FIELD",
        "Declares a field's type, default, assertion, references. See https://docs.tessaridb.com/query-language/definitions",
    ),
    (
        "DEFINE INDEX",
        "Declares an access path: value, unique, composite, CONTAINS, or SEARCH. See https://docs.tessaridb.com/query-language/definitions",
    ),
    (
        "DEFINE EVENT",
        "Runs statements after writes on a table, in the writer's transaction. See https://docs.tessaridb.com/query-language/events",
    ),
    (
        "DEFINE GRAPH",
        "Declares a graph so adjacency sits beside the node. See https://docs.tessaridb.com/query-language/graphs",
    ),
    (
        "DEFINE USER",
        "Declares a user with a role or authority set and a credential. See https://docs.tessaridb.com/security/users",
    ),
    (
        "DEFINE VAULT",
        "Declares a store for secrets the database cannot read back. See https://docs.tessaridb.com/query-language/vaults",
    ),
    (
        "DEFINE QUEUE",
        "Declares a queue: one record to exactly one worker, with a hold that lapses. See https://docs.tessaridb.com/query-language/queues",
    ),
    (
        "DEFINE TOPIC",
        "Declares a commit-ordered stream every reader paces itself over. See https://docs.tessaridb.com/query-language/topics",
    ),
    (
        "DEFINE SERIES",
        "Declares time-ordered events under a declared floor. See https://docs.tessaridb.com/query-language/time-series",
    ),
    (
        "DEFINE VIEW",
        "Declares a named read that runs with the caller's permissions. See https://docs.tessaridb.com/query-language/views",
    ),
    (
        "ALTER",
        "Amends a declared object; the named clause replaces, an absent clause leaves alone. See https://docs.tessaridb.com/query-language/definitions",
    ),
    (
        "DROP",
        "Removes a declared object. See https://docs.tessaridb.com/query-language/definitions",
    ),
    (
        "USE",
        "Selects the namespace and database this session reads and writes. See https://docs.tessaridb.com/query-language/bindings",
    ),
    (
        "INFO",
        "Asks the catalog or node what it actually holds. See https://docs.tessaridb.com/reference/statements",
    ),
    (
        "BEGIN",
        "Opens a transaction; everything until COMMIT or CANCEL is one snapshot. See https://docs.tessaridb.com/query-language/transactions",
    ),
    (
        "COMMIT",
        "Commits the open transaction; ACROSS LEADERS settles a multi-leader write in one round. See https://docs.tessaridb.com/query-language/transactions",
    ),
    (
        "CANCEL",
        "Discards the open transaction and everything in it. See https://docs.tessaridb.com/query-language/transactions",
    ),
    (
        "SAVEPOINT",
        "Marks a point inside a transaction ROLLBACK TO can return to. See https://docs.tessaridb.com/query-language/transactions",
    ),
    (
        "LET",
        "Binds a parameter for the statements that follow it. See https://docs.tessaridb.com/query-language/bindings",
    ),
    (
        "RETURN",
        "Answers with a value; the last word of a script. See https://docs.tessaridb.com/query-language/bindings",
    ),
    (
        "THROW",
        "Refuses with a message, in the writer's own words. See https://docs.tessaridb.com/query-language/expressions",
    ),
    (
        "GRANT",
        "Gives a user an authority over a reach — the whole story of what they may do. See https://docs.tessaridb.com/security/grants",
    ),
    (
        "REVOKE",
        "Takes an authority back. See https://docs.tessaridb.com/security/grants",
    ),
    (
        "BACKUP",
        "Takes the store away: snapshot by default, the log or a script when named. See https://docs.tessaridb.com/operations/backup-and-restore",
    ),
    (
        "CHECK",
        "CHECK TABLE re-reads every stored record against what the table now declares. See https://docs.tessaridb.com/reference/statements",
    ),
    (
        "ANALYZE",
        "ANALYZE TABLE refreshes the per-field statistics the planner ranks on. See https://docs.tessaridb.com/query-language/what-a-read-reports",
    ),
    (
        "REBUILD",
        "REBUILD INDEX rebuilds an access path, measuring what approximation uses. See https://docs.tessaridb.com/query-language/vectors",
    ),
    (
        "EXPLAIN",
        "Shows the plan a read would run, without running it. See https://docs.tessaridb.com/query-language/what-a-read-reports",
    ),
    (
        "FROM",
        "Names what a read walks: tables, records, traversals, subqueries. See https://docs.tessaridb.com/query-language/documents",
    ),
    (
        "WHERE",
        "The condition a record must satisfy; absence narrows, a wrong type refuses. See https://docs.tessaridb.com/query-language/conditions",
    ),
    (
        "ORDER BY",
        "Sorts the answer; FUSE ranks text, vector and distance together by place, never by adding scores. See https://docs.tessaridb.com/query-language/several-orders-at-once",
    ),
    (
        "GROUP BY",
        "Folds records into groups; HAVING is read once per group, after the folds. See https://docs.tessaridb.com/query-language/conditions",
    ),
    (
        "LIMIT",
        "Caps how many records answer; behind a WHERE it stops where the answer fills. See https://docs.tessaridb.com/query-language/paging-and-history",
    ),
    (
        "MATCHES",
        "Full-text search over an analysed field, with PREFIX / INFIX / FUZZY modifiers. See https://docs.tessaridb.com/query-language/full-text-search",
    ),
    (
        "CONTAINS",
        "Asks whether an array holds a value or a document holds a sub-document. See https://docs.tessaridb.com/query-language/conditions",
    ),
];

/// Hover text for a word: the curated table first, then a family line for any
/// catalogued function, so every completed word also explains itself.
#[must_use]
pub fn hover_doc(word: &str) -> Option<&'static str> {
    if let Some((_, text)) = HOVER_DOCS.iter().find(|(key, _)| *key == word) {
        return Some(text);
    }
    if let Some((family, _)) = word.split_once("::") {
        if FUNCTIONS.contains(&word) {
            return Some(match family {
                "array" => {
                    "Array function — folds, order and shape over arrays. See https://docs.tessaridb.com/reference/functions"
                }
                "crypto" => {
                    "Digest function — fingerprints, not password hashing. See https://docs.tessaridb.com/reference/functions"
                }
                "duration" => {
                    "Duration function — spans between instants. See https://docs.tessaridb.com/reference/functions"
                }
                "encoding" => {
                    "Encoding function — text forms of bytes and back. See https://docs.tessaridb.com/reference/functions"
                }
                "geo" => {
                    "Geometry function — shapes, predicates and distances in metres. See https://docs.tessaridb.com/query-language/geometry"
                }
                "math" => {
                    "Math function — arithmetic over numbers. See https://docs.tessaridb.com/reference/functions"
                }
                "object" => {
                    "Object function — fields and entries of documents. See https://docs.tessaridb.com/reference/functions"
                }
                "rand" => {
                    "Random function — values from the generator. See https://docs.tessaridb.com/reference/functions"
                }
                "search" => {
                    "Search function — scores, highlights and snippets over a SEARCH index. See https://docs.tessaridb.com/query-language/full-text-search"
                }
                "session" => {
                    "Session function — what this connection holds. See https://docs.tessaridb.com/reference/functions"
                }
                "set" => {
                    "Set function — membership over set-typed fields. See https://docs.tessaridb.com/reference/functions"
                }
                "string" => {
                    "String function — characters, not bytes. See https://docs.tessaridb.com/reference/functions"
                }
                "time" => {
                    "Time function — instants, buckets and calendars in UTC. See https://docs.tessaridb.com/reference/functions"
                }
                "type" => {
                    "Type function — conversion and inspection between types. See https://docs.tessaridb.com/reference/functions"
                }
                "vector" => {
                    "Vector function — distances and arithmetic over fixed-width vectors. See https://docs.tessaridb.com/query-language/vectors"
                }
                "scores" => {
                    "Scores function — combining and inspecting ranked scores. See https://docs.tessaridb.com/reference/functions"
                }
                "json" => {
                    "JSON function — parsing and encoding JSON text. See https://docs.tessaridb.com/reference/functions"
                }
                _ => "TessariQL function. See https://docs.tessaridb.com/reference/functions",
            });
        }
    }
    None
}
