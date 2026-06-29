use axum::Router;
use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientRequest, Request, ServerResult},
    transport::StreamableHttpClientTransport,
};
use serde_json::json;
use tokio::net::TcpListener;

#[derive(Clone, Default)]
struct TestClient;

impl ClientHandler for TestClient {}

async fn spawn_app() -> (String, tokio::task::JoinHandle<()>) {
    let app: Router = ruminate::http::app();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/mcp"), handle)
}

async fn connect(url: &str) -> rmcp::service::RunningService<rmcp::RoleClient, TestClient> {
    let transport = StreamableHttpClientTransport::from_uri(url.to_string());
    TestClient.serve(transport).await.unwrap()
}

async fn call(
    client: &rmcp::service::RunningService<rmcp::RoleClient, TestClient>,
    name: &str,
    args: serde_json::Value,
) -> serde_json::Value {
    let response = client
        .send_request(ClientRequest::CallToolRequest(Request::new(
            CallToolRequestParams::new(name.to_string())
                .with_arguments(args.as_object().cloned().unwrap_or_default()),
        )))
        .await
        .unwrap();

    let ServerResult::CallToolResult(result) = response else {
        panic!("unexpected response: {response:?}");
    };
    result.structured_content.unwrap()
}

#[tokio::test]
async fn mcp_sessions_are_isolated() {
    let (url, handle) = spawn_app().await;
    let first = connect(&url).await;
    let second = connect(&url).await;

    let first_result = call(
        &first,
        "ruminate",
        json!({
            "thought": "first",
            "thoughtNumber": 1,
            "totalThoughts": 3,
            "nextThoughtNeeded": true
        }),
    )
    .await;
    let second_result = call(
        &second,
        "ruminate",
        json!({
            "thought": "second",
            "thoughtNumber": 1,
            "totalThoughts": 3,
            "nextThoughtNeeded": true
        }),
    )
    .await;

    assert_eq!(first_result["thoughtHistoryLength"], 1);
    assert_eq!(second_result["thoughtHistoryLength"], 1);
    first.cancel().await.unwrap();
    second.cancel().await.unwrap();
    handle.abort();
}

#[tokio::test]
async fn workflow_records_are_session_isolated() {
    let (url, handle) = spawn_app().await;
    let first = connect(&url).await;
    let second = connect(&url).await;

    call(
        &first,
        "ruminate_note",
        json!({
            "kind": "decision",
            "text": "Rust implementation",
            "tags": ["migration"]
        }),
    )
    .await;

    let first_summary = call(&first, "ruminate_inspect", json!({ "view": "summary" })).await;
    let second_summary = call(&second, "ruminate_inspect", json!({ "view": "summary" })).await;

    assert_eq!(first_summary["notes"], 1);
    assert_eq!(second_summary["notes"], 0);
    first.cancel().await.unwrap();
    second.cancel().await.unwrap();
    handle.abort();
}
