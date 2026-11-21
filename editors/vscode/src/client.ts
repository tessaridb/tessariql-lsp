// The VS Code client: spawn tessariql-lsp over stdio and let
// vscode-languageclient speak LSP to it. The server path and the optional
// node address come from the extension configuration; the node address is
// passed as TESSARIQL_LSP_NODE because that is the server's feature flag.

import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration("tessariql");

  const run: ServerOptions = {
    command: config.get<string>("serverPath", "tessariql-lsp"),
    args: [],
    options: {
      env: {
        ...process.env,
        ...(config.get<string>("serverNode", "")
          ? { TESSARIQL_LSP_NODE: config.get<string>("serverNode", "") }
          : {}),
      },
    },
  };

  const clientOptions: LanguageClientOptions = {
    // The language id editors give .tessariql buffers; documents with this id
    // drive the server. Grammar-based highlighting ships from a tree-sitter
    // package later — the server owns analysis, not colors.
    documentSelector: [{ scheme: "file", language: "tessariql" }],
  };

  client = new LanguageClient("tessariql", "TessariQL", run, clientOptions);
  void client.start();

  context.subscriptions.push(
    vscode.commands.registerCommand("tessariql.runStatement", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !client) {
        return;
      }
      const uri = editor.document.uri.toString();
      const position = editor.selection.active;
      try {
        const result: unknown = await client.sendRequest("workspace/executeCommand", {
          command: "tessariql.runStatement",
          arguments: [uri, position.line, position.character],
        });
        if (typeof result === "string") {
          await vscode.window.showInformationMessage(result, { modal: false });
        }
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        await vscode.window.showErrorMessage(`tessariql: ${message}`);
      }
    }),
  );
}

export async function deactivate(): Promise<void> {
  if (client) {
    await client.stop();
    client = undefined;
  }
}
