use std::{collections::BTreeSet, process::Stdio, time::Duration};

use serde_json::{Value, json};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{ChildStdin, ChildStdout, Command},
    time::timeout,
};

async fn write_message(stdin: &mut ChildStdin, message: Value) {
    let message = serde_json::to_string(&message).expect("MCP request should serialize");
    stdin
        .write_all(message.as_bytes())
        .await
        .expect("binary stdin should accept MCP request");
    stdin
        .write_all(b"\n")
        .await
        .expect("binary stdin should accept MCP delimiter");
    stdin.flush().await.expect("binary stdin should flush");
}

async fn read_response(lines: &mut Lines<BufReader<ChildStdout>>) -> Value {
    let line = timeout(Duration::from_secs(2), lines.next_line())
        .await
        .expect("server should respond over stdio")
        .expect("server stdout should be readable")
        .expect("server should not close stdio unexpectedly");
    serde_json::from_str(&line).expect("stdout must contain only JSON-RPC messages")
}

#[tokio::test]
async fn binary_stdio_initializes_and_lists_the_six_ruminate_tools() {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ruminate"));
    command
        .env("PORT", "0")
        .env("RUMINATE_TRANSPORT", "stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().expect("binary should start");
    let mut stdin = child.stdin.take().expect("binary stdin should be piped");
    let stdout = child.stdout.take().expect("binary stdout should be piped");
    let mut lines = BufReader::new(stdout).lines();

    write_message(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": { "name": "stdio-contract", "version": "1.0" }
            }
        }),
    )
    .await;
    let initialize = read_response(&mut lines).await;
    assert_eq!(initialize["id"], 1);
    assert_eq!(initialize["result"]["serverInfo"]["name"], "ruminate");

    write_message(
        &mut stdin,
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
    )
    .await;
    write_message(
        &mut stdin,
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {} }),
    )
    .await;
    let tools = read_response(&mut lines).await;

    let names: BTreeSet<_> = tools["result"]["tools"]
        .as_array()
        .expect("tools/list result should contain tools")
        .iter()
        .map(|tool| {
            tool["name"]
                .as_str()
                .expect("tool should have a name")
                .to_owned()
        })
        .collect();
    assert_eq!(
        names,
        BTreeSet::from([
            "ruminate".into(),
            "ruminate_note".into(),
            "ruminate_checkpoint".into(),
            "ruminate_gate".into(),
            "ruminate_inspect".into(),
            "ruminate_reflect".into(),
        ])
    );

    stdin.shutdown().await.expect("binary stdin should close");
    drop(stdin);
    assert!(
        timeout(Duration::from_secs(2), child.wait())
            .await
            .expect("binary should stop after stdin closes")
            .expect("binary process should be waitable")
            .success(),
        "binary should exit successfully"
    );
}
