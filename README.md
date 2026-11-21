<div align="center">

<img src="assets/logo/tessaridb-mark-256.png" alt="" width="88" height="88">

# TessariQL language server (tessariql-lsp)

**The [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
server for [TessariQL](https://docs.tessaridb.com)** — the one query language of
[TessariDB](https://github.com/tessaridb/tessaridb) — so every editor gets the same
completion, hover and diagnostics the language deserves.

[![status](https://img.shields.io/badge/status-in%20development-D98E33?style=flat-square)](#status)
[![licence](https://img.shields.io/badge/licence-Apache--2.0-6B5FD1?style=flat-square)](LICENSE)
[![lsp](https://img.shields.io/badge/LSP-3.17-6B5FD1?style=flat-square)](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)

</div>

> [!NOTE]
> **In early development.** The server speaks the protocol, syncs documents,
> and serves syntax diagnostics, completion and hover — see
> [**Status**](#status). Checked against **TessariDB 0.50.0**.

Licence: **Apache-2.0**. A language server is infrastructure for everyone who writes
the language, so it is permissively licensed on purpose — the database server is
licensed separately, and that licence reaches neither this server nor anything
written from it.

---

## Why this repository exists separately

Editor support is written *against a language*, not against a database build. If
completion and diagnostics lived inside the server repository, every editor
integration would inherit a database release cycle, a database's build, and a
database's licence. The language server stands on its own:

- it speaks one protocol, so VS Code, Neovim, Helix, Zed and everything else with an
  LSP client share one implementation;
- **it never depends on the database's source** — the grammar here is written from
  the published language and checked against a statement corpus, so the server's
  licence stays clean and the grammar is validated, not assumed;
- a behaviour the server needs and the language docs do not state is a gap in the
  docs, filed there first.

## What is here

| path | what it is |
|---|---|
| `crates/` | the workspace: `tessariql-lsp-constants`, `tessariql-lsp-types`, `tessariql-lsp-core` (open documents), `tessariql-lsp-observability` (tracing to stderr — stdout is the protocol stream), `tree-sitter-tessariql` (the grammar: C parser + Rust binding, error-tolerant by construction), `tessariql-lsp` (the binary: LSP adapter + composition root) |
| `docs/DESIGN.md` | the design record — stack choice, grammar strategy, milestones, structure worksheet |
| `Dockerfile` | two-stage build; the image runs `tessariql-lsp` with stdio as its only interface — no ports on purpose |
| `corpus/` | 172 engine-validated TessariQL statements (tessaridb 0.50.0) the grammar is checked against — every statement parses with zero ERROR nodes, and mutation smoke over the corpus never panics the parser |

## Status

| milestone | what it brings | state |
|---|---|---|
| M1 | LSP lifecycle over stdio, document sync | done |
| M2 | resilient TessariQL grammar (tree-sitter), syntax diagnostics | done |
| M3 | completion — keywords, statements, functions | done |
| M4 | hover — keywords, statements, functions with doc links | done |
| M5 | catalog-aware completion and run-statement under cursor, behind a connection config |
| M6 | packaging — VS Code extension, install notes for other editors |

## Development

```sh
cargo build
cargo test            # includes the corpus conformance suite and the mutation smoke
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

The grammar lives at `crates/tree-sitter-tessariql/grammar.js`; regenerate the
C parser after editing it with `npx -y tree-sitter-cli@0.25.10 generate --abi 14`
run from that crate's directory. The corpus is regenerated against a live node
per engine release (see `docs/DESIGN.md`).

```sh
docker build -t tessaridb/tessariql-lsp:dev .
```

Branch `dev` is where work happens; `main` moves only by a release merge, and
releases are tagged on `main`. Manifests state their own version. The toolchain
channel is `stable` (see `rust-toolchain.toml`). The Docker Hub repository is
`tessaridb/tessariql-lsp`; publishing an image is a release act.

## Links

- Documentation: https://docs.tessaridb.com
- The database: https://github.com/tessaridb/tessaridb
- The protocol (for client authors): https://github.com/tessaridb/tessaridb-protocol
