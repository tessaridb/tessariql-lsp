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
