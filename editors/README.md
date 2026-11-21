# Editor integration

`tessariql-lsp` speaks the Language Server Protocol over stdio — any editor
with an LSP client can run it. This folder holds the VS Code extension; the
snippets below cover the common clients.

The server reads its node connection from the environment (see the
repository README): without `TESSARIQL_LSP_NODE` it serves diagnostics,
vocabulary completion and hover; with it, catalog-aware completion and the
`runStatement` command join in.

## VS Code

Build the extension and install the artifact:

```sh
cd editors/vscode
npm install
npm run compile
npx @vscode/vsce package          # → tessariql-lsp.vsix
code --install-extension tessariql-lsp.vsix
```

Set `tessariql.serverPath` when the binary is not on `PATH`, and
`tessariql.serverNode` to enable the catalog and the run command.

## Neovim (lspconfig)

```lua
require("lspconfig.configs").tessariql = {
  default_config = {
    cmd = { "tessariql-lsp" },
    filetypes = { "tessariql" },
    root_dir = function(fname)
      return vim.fs.dirname(vim.fs.find({ ".git" }, { path = fname, upward = true })[1])
    end,
    settings = {},
  },
}
require("lspconfig").tessariql.setup({})
```

## Helix (`languages.toml`)

```toml
[language-server.tessariql]
command = "tessariql-lsp"

[[language]]
name = "tessariql"
language-servers = ["tessariql"]
# file-type detection is yours to declare (e.g. extension = ["tessariql"])
```

## Zed (`settings.json`)

```json
{
  "lsp": {
    "tessariql-lsp": {
      "binary": { "path": "tessariql-lsp" }
    }
  }
}
```

## The run command in generic clients

Any client can send `workspace/executeCommand` with
`{ "command": "tessariql.runStatement", "arguments": [uri, line, character] }`
to run the statement under the cursor; the server answers with the node's
reply as a string, or an error carrying the node's own refusal text.
