# TessariQL for VS Code

Language support for [TessariQL](https://docs.tessaridb.com) through the
`tessariql-lsp` language server: syntax diagnostics from a tree-sitter
grammar, keyword / statement / function completion, hover with docs links,
and — with a node configured — catalog-aware completion and a run-statement
-under-cursor command.

## Setup

1. Build or install the server binary (`cargo install --path
   crates/tessariql-lsp` from the repository, or use the Docker image).
2. Set `tessariql.serverPath` if the binary is not on `PATH`.
3. Optionally set `tessariql.serverNode` to a `host:port` TessariDB node —
   this enables catalog completion and the run command. Namespace, database,
   and credentials are read from the environment by the server
   (`TESSARIQL_LSP_NAMESPACE`, `TESSARIQL_LSP_DATABASE`, `TESSARIQL_LSP_USER`,
   `TESSARIQL_LSP_PASSWORD`).

## Commands

- **TessariQL: Run statement under cursor** (`ctrl+alt+enter`, macOS
  `cmd+alt+enter`) — runs the statement under the cursor on the configured
  node and shows its answer.

Install a locally built copy with `code --install-extension tessariql-lsp.vsix`.
