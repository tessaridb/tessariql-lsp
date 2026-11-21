# Design record — tessariql-lsp

The design decisions behind the TessariQL language server, with the sources they
rest on (retrieved 2026-11-21). Checked against TessariDB 0.50.0.

## 1. The protocol

LSP is JSON-RPC 3.0 over stdio with `Content-Length` framing; the server
advertises capabilities in `initialize` and answers requests, publishes
diagnostics, and reacts to document-sync notifications. Spec:
<https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/>

## 2. Stack research

| option | verdict |
|---|---|
| `tower-lsp` (original) | unmaintained ~3 years — rejected for a new project |
| **`tower-lsp-server`** (tower-lsp-community fork) | **chosen**: actively maintained (0.23.x, 2026), tower-lsp-compatible API, async, native `async fn` traits |
| `lsp-server` (rust-analyzer) | minimal sync core; rejected — we want the ergonomic async surface, and rust-analyzer's own needs (incremental, salsa) are heavier than ours |
| `async-lsp` | leaner but far less adopted; rejected |

Sources: <https://github.com/tower-lsp-community/tower-lsp-server> ·
<https://github.com/ebkalderon/tower-lsp> ·
<https://lib.rs/crates/lsp-server> ·
<https://lib.rs/crates/async-lsp> ·
<https://codeinput.com/blog/lsp-server> (2026 tutorial recommending the fork).

Runtime: tokio multi-thread (workspace standard). Tracing to **stderr** — stdout
is the protocol stream.

## 3. The hard problem: parsing code that doesn't parse yet

A language server constantly sees *invalid* text — half-typed statements,
unbalanced parentheses. The reference architecture is Supabase's Postgres
language server (pgt / postgres_lsp), written in Rust:

- a **strict parser** (`libpg_query`, the real PostgreSQL parser) gives precise
  ASTs and diagnostics for valid code — but refuses the whole document on any
  syntax error;
- an **error-tolerant parser** (a hand-written resilient parser, later aided by
  tree-sitter) always produces a tree, which is what completion and mid-typing
  highlighting need;
- a **schema cache** feeds catalog-aware completion;
- features are separate crates (`pg_completions`, `pg_hover`, `pg_lint`, …).

Sources: <https://github.com/supabase-community/postgres_lsp/blob/main/ARCHITECTURE.md> ·
<https://supabase.com/blog/postgres-language-server-implementing-parser> ·
<https://supabase.com/blog/postgres-language-server>

For TessariQL the strict parser lives in the engine (`tessari-ql`, BUSL-1.1) and
**must not become a dependency** of this Apache-2.0 repository (org hygiene: the
SDK clients never depend on the engine by any mechanism, and a dev tool
shouldn't either — the licence would ride along). So the plan inverts the
reference slightly:

- **M2**: a resilient TessariQL grammar as a tree-sitter grammar
  (`tree-sitter-tessariql`, C, compiled with `cc`) — always produces a tree,
  incremental, and reusable later by editors for syntax highlighting.
  Keywords are uppercase in the grammar with the `word:` identifier
  extraction (the language's written convention; SG4 completion will offer
  uppercase keywords — extending to case-insensitive keywords is recorded
  work if the engine's lexer folding ever matters to an editor);
- **corpus/**: statements collected from the engine's docs, changelog and
  worked examples (0.50.0), parsed both by the grammar here and executed by a
  real node — the grammar is *validated against the engine*, never assumed;
- diagnostics use the grammar's error nodes first; statement-level semantic
  checks arrive when the grammar stabilises.

## 4. Structure worksheet

Project prefix `tessariql`, kind application.

| trigger (canon M5) | present | crate |
|---|---|---|
| application | yes | `tessariql-lsp-constants`, `tessariql-lsp-types`, `tessariql-lsp-core`, binary `tessariql-lsp` |
| long-running process | yes | `tessariql-lsp-observability` (tracing to stderr) |
| persistence | no | — (none; documents are session state) |
| network API | no | — (LSP-over-stdio is the only surface) |
| background/queue work | no | — |
| external client (node connection, M2+) | deferred | `tessariql-infrastructure` arrives with M5 |

Recorded decision — the LSP adapter (protocol → domain translation) lives in the
binary crate (`crates/tessariql-lsp/src/adapter.rs`) rather than an `api` crate,
because the canon's `api` role is HTTP/gRPC-shaped (routes/middleware/guards) and
a second runtime surface does not exist. If one ever appears, the transport
splits into its own crate then — not before.

Edges (canon M6): core → types; observability → constants; binary → any library.
No storage, no I/O in core; the domain is synchronous and the adapter keeps
every domain call a short synchronous section with no guard across `.await`.

Conformance: `rust_structure_check.py --kind application --prefix tessariql-lsp`
exit 0 on the M1 tree (32 rules pass). Internal crates carry the
`tessariql-lsp-` prefix (owner 2026-11-21) so no crate name collides when
individual crates are published later; the publishable units are the binary
crate (`tessariql-lsp`) and the grammar crate (`tree-sitter-tessariql`).
The grammar crate carries a WAIVED structure rule: `tree-sitter-<lang>` is the
ecosystem-mandated name (editor and highlighter discovery assumes it), so
RS-SET-NAMING is waived as
`--waive RS-SET-NAMING:tree-sitter-<lang> is the ecosystem grammar-crate name (§3)`.

## 5. Milestones

See README §Status. M1 (this skeleton) = lifecycle over stdio + document sync
with a monotonic-version contract, `cargo build`/`clippy -D warnings`/`test`
green, structure check exit 0. M2 = tree-sitter grammar + corpus + syntax
diagnostics. M3/M4 = completion/hover. M5 = catalog-aware completion + run
statement under cursor (introduces `tessariql-infrastructure` and the
`tessaridb-client` connection). M6 = packaging (VS Code extension; the Docker
image `tessaridb/tessariql-lsp` builds from the root Dockerfile).
