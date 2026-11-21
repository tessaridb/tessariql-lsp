//! Lifecycle round-trip over real stdio: spawn the built binary, speak the
//! base protocol with Content-Length framing, and assert the answers.
//!
//! The two-sided rule this test pins: a green run proves the binary speaks the
//! protocol (the only state that makes it pass), and a framing/capability
//! regression makes it fail — both states are reachable by editing the
//! adapter.

use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

/// Writes one base-protocol message.
fn write_message(stdin: &mut ChildStdin, body: &Value) {
    let bytes = serde_json::to_vec(body).expect("json serializes");
    write!(stdin, "Content-Length: {}\r\n\r\n", bytes.len()).expect("write header");
    stdin.write_all(&bytes).expect("write body");
    stdin.flush().expect("flush");
}

/// Reads exactly one message, parsing the header block first.
fn read_message(reader: &mut BufReader<ChildStdout>) -> Value {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line).expect("read header line");
        assert!(read > 0, "server closed the stream mid-conversation");
        let line = line.trim_end();
        if line.is_empty() {
            break; // end of the header block
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = Some(value.trim().parse::<usize>().expect("numeric length"));
        }
    }
    let length = content_length.expect("every LSP message carries Content-Length");
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).expect("read body");
    serde_json::from_slice(&body).expect("body is json")
}

/// The spawned server. The reader owns the stdout pipe (taken from the child)
/// so framing state survives across responses — a fresh BufReader per call
/// would silently drop buffered bytes between messages.
struct Server {
    child: Child,
    reader: BufReader<ChildStdout>,
}

impl Server {
    fn spawn() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_tessariql-lsp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("binary spawns");
        let stdout = child.stdout.take().expect("stdout pipe");
        Self {
            child,
            reader: BufReader::new(stdout),
        }
    }

    fn request(&mut self, id: i64, method: &str, params: Value) {
        let stdin = self.child.stdin.as_mut().expect("stdin pipe");
        write_message(
            stdin,
            &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
        );
    }

    fn notify(&mut self, method: &str, params: Value) {
        let stdin = self.child.stdin.as_mut().expect("stdin pipe");
        write_message(
            stdin,
            &json!({"jsonrpc": "2.0", "method": method, "params": params}),
        );
    }

    fn response(&mut self, want_id: i64) -> Value {
        for _ in 0..8 {
            let message = read_message(&mut self.reader);
            if message.get("id").and_then(Value::as_i64) == Some(want_id) {
                return message;
            }
            // A notification (e.g. window/logMessage) may interleave; skip it.
        }
        panic!("no response for request {want_id} within 8 messages");
    }

    /// Reads until a notification with the wanted method arrives, skipping
    /// interleaved messages (log messages, other notifications).
    fn notification(&mut self, want_method: &str) -> Value {
        for _ in 0..8 {
            let message = read_message(&mut self.reader);
            if message.get("method").and_then(Value::as_str) == Some(want_method) {
                return message;
            }
        }
        panic!("no {want_method} notification within 8 messages");
    }

    fn wait(mut self) -> bool {
        self.child.wait().expect("wait").success()
    }
}

#[test]
fn initialize_shutdown_exit_round_trip() {
    let mut server = Server::spawn();

    server.request(
        1,
        "initialize",
        json!({"processId": null, "rootUri": null, "capabilities": {}}),
    );
    let initialize = server.response(1);
    let capabilities = &initialize["result"]["capabilities"];
    // FULL sync kind is the number 1; the server advertises nothing else yet.
    assert_eq!(capabilities["textDocumentSync"], json!(1));
    assert_eq!(
        initialize["result"]["serverInfo"]["name"],
        json!("tessariql-language-server")
    );

    server.notify("initialized", json!({}));

    server.request(2, "shutdown", json!(null));
    let shutdown = server.response(2);
    assert!(
        shutdown.get("error").is_none(),
        "shutdown failed: {shutdown}"
    );
    assert_eq!(shutdown["result"], Value::Null);

    server.notify("exit", json!({}));
    assert!(server.wait(), "process exits cleanly after exit");
}

/// Syntax diagnostics end to end: an unparseable statement is published on
/// open, republished clean when the change fixes it. The two-sided rule: a
/// server that never runs the grammar never publishes (the only state that
/// fails this), and a grammar regression makes the broken document pass —
/// both states are reachable by editing the adapter or the grammar.
#[test]
fn syntax_diagnostics_publish_on_open_and_clear_on_change() {
    let mut server = Server::spawn();
    server.request(
        1,
        "initialize",
        json!({"processId": null, "rootUri": null, "capabilities": {}}),
    );
    server.response(1);
    server.notify("initialized", json!({}));

    let uri = "file:///broken.tessariql";
    server.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "tessariql",
                "version": 1,
                "text": "SELEC * FROM people;"
            }
        }),
    );
    let published = server.notification("textDocument/publishDiagnostics");
    assert_eq!(published["params"]["uri"], json!(uri));
    let diagnostics = published["params"]["diagnostics"]
        .as_array()
        .expect("diagnostics is an array");
    assert_eq!(
        diagnostics.len(),
        1,
        "one statement, one error: {diagnostics:?}"
    );
    assert_eq!(diagnostics[0]["message"], json!("syntax error"));
    assert_eq!(diagnostics[0]["severity"], json!(1));
    assert_eq!(diagnostics[0]["range"]["start"]["line"], json!(0));

    server.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 2},
            "contentChanges": [{"text": "SELECT * FROM people;"}]
        }),
    );
    let cleared = server.notification("textDocument/publishDiagnostics");
    assert_eq!(
        cleared["params"]["diagnostics"],
        json!([]),
        "a correct statement republishes empty diagnostics"
    );

    server.request(9, "shutdown", json!(null));
    server.response(9);
    server.notify("exit", json!({}));
    assert!(server.wait(), "process exits cleanly after exit");
}

/// Completion over the wire: a half-typed keyword completes to the keyword
/// catalog, and a namespace prefix completes to the function catalog with
/// the FUNCTION kind. Two-sided: a server that never looks at the catalogs
/// answers empty (fails), and both catalogs regressing fails too.
#[test]
fn completion_offers_keywords_and_functions() {
    const KEYWORD: i64 = 14;
    const FUNCTION: i64 = 3;
    let mut server = Server::spawn();
    server.request(
        1,
        "initialize",
        json!({"processId": null, "rootUri": null, "capabilities": {}}),
    );
    let initialize = server.response(1);
    assert!(
        initialize["result"]["capabilities"]["completionProvider"].is_object(),
        "the server advertises completion"
    );
    server.notify("initialized", json!({}));

    let uri = "file:///complete.tessariql";
    let open = |server: &mut Server, version: i64, text: &str| {
        server.notify(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": "tessariql",
                    "version": version,
                    "text": text
                }
            }),
        );
    };
    let complete_at = |server: &mut Server, id: i64, character: usize| {
        server.request(
            id,
            "textDocument/completion",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": 0, "character": character}
            }),
        );
        server.response(id)["result"]
            .as_array()
            .expect("completion result is an array")
            .clone()
    };

    open(&mut server, 1, "SEL");
    let keyword_hits = complete_at(&mut server, 2, 3);
    let select = keyword_hits
        .iter()
        .find(|item| item["label"] == json!("SELECT"))
        .expect("SELECT is offered for the SEL prefix");
    assert_eq!(select["kind"], json!(KEYWORD));

    open(&mut server, 2, "SELECT string::up FROM people;");
    // wait: didOpen for an already-open document is ignored; use didChange.
    server.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": uri, "version": 2},
            "contentChanges": [{"text": "SELECT string::up FROM people;"}]
        }),
    );
    let function_hits = complete_at(&mut server, 3, 17);
    let upper = function_hits
        .iter()
        .find(|item| item["label"] == json!("string::upper"))
        .expect("string::upper is offered for the string::up prefix");
    assert_eq!(upper["kind"], json!(FUNCTION));
    assert!(
        function_hits.iter().all(|item| item["label"]
            .as_str()
            .is_some_and(|label| label.starts_with("string::"))),
        "a namespace prefix only offers functions: {function_hits:?}"
    );

    server.request(9, "shutdown", json!(null));
    server.response(9);
    server.notify("exit", json!({}));
    assert!(server.wait(), "process exits cleanly after exit");
}

/// Hover over the wire: hovering a keyword answers with its docs line, and
/// hovering a function answers with the family text. Two-sided: no lookup
/// ever wired answers null (fails the first assert), and the docs table
/// losing the entry fails it too.
#[test]
fn hover_explains_keywords_and_functions() {
    let mut server = Server::spawn();
    server.request(
        1,
        "initialize",
        json!({"processId": null, "rootUri": null, "capabilities": {}}),
    );
    server.response(1);
    server.notify("initialized", json!({}));

    let uri = "file:///hover.tessariql";
    server.notify(
        "textDocument/didOpen",
        json!({
            "textDocument": {
                "uri": uri,
                "languageId": "tessariql",
                "version": 1,
                "text": "SELECT string::upper(name) FROM people;"
            }
        }),
    );

    let hover_at = |server: &mut Server, id: i64, character: usize| -> Value {
        server.request(
            id,
            "textDocument/hover",
            json!({
                "textDocument": {"uri": uri},
                "position": {"line": 0, "character": character}
            }),
        );
        server.response(id)["result"].clone()
    };

    // Cursor on SELECT (character 2).
    let keyword = hover_at(&mut server, 2, 2);
    // MarkedString::String serializes as a bare JSON string.
    let contents = keyword["contents"]
        .as_str()
        .expect("hover contents is a string");
    assert!(
        contents.starts_with("Reads records"),
        "SELECT hover explains the statement: {contents}"
    );
    assert!(
        contents.contains("docs.tessaridb.com"),
        "the hover points at the docs: {contents}"
    );

    // Cursor inside `string::upper` (character 12).
    let function = hover_at(&mut server, 3, 12);
    let contents = function["contents"]
        .as_str()
        .expect("hover contents is a string");
    assert!(
        contents.contains("String function"),
        "function hover explains the family: {contents}"
    );

    server.request(9, "shutdown", json!(null));
    server.response(9);
    server.notify("exit", json!({}));
    assert!(server.wait(), "process exits cleanly after exit");
}
