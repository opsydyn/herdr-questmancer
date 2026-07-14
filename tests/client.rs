use std::path::{Path, PathBuf};

use herdr_webmaster::herdr::client::{ClientError, HerdrClient};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};

fn listener() -> (TempDir, PathBuf, UnixListener) {
    let directory = tempfile::tempdir().expect("temporary socket directory");
    let path = directory.path().join("herdr.sock");
    let listener = UnixListener::bind(&path).expect("bind fake Herdr socket");
    (directory, path, listener)
}

async fn read_request(stream: &mut UnixStream) -> Value {
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .await
        .expect("request line");
    serde_json::from_str(&line).expect("request JSON")
}

async fn write_response(stream: &mut UnixStream, response: &Value) {
    stream
        .write_all(serde_json::to_string(response).unwrap().as_bytes())
        .await
        .unwrap();
    stream.write_all(b"\n").await.unwrap();
}

fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/herdr")
        .join(name);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

#[tokio::test]
async fn each_ordinary_request_uses_a_new_connection() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut ping_stream, _) = listener.accept().await.unwrap();
        let ping_request = read_request(&mut ping_stream).await;
        assert_eq!(ping_request["method"], "ping");
        let mut pong = fixture("pong.json");
        pong["id"] = ping_request["id"].clone();
        write_response(&mut ping_stream, &pong).await;
        drop(ping_stream);

        let (mut snapshot_stream, _) = listener.accept().await.unwrap();
        let snapshot_request = read_request(&mut snapshot_stream).await;
        assert_eq!(snapshot_request["method"], "session.snapshot");
        let mut snapshot = fixture("session_snapshot.json");
        snapshot["id"] = snapshot_request["id"].clone();
        write_response(&mut snapshot_stream, &snapshot).await;
    });
    let client = HerdrClient::new(path);

    assert_eq!(client.ping().await.unwrap().protocol, 16);
    assert_eq!(client.snapshot().await.unwrap().agents.len(), 1);
    server.await.unwrap();
}

#[tokio::test]
async fn rejects_a_mismatched_response_id() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let _request = read_request(&mut stream).await;
        let response = fixture("pong.json");
        write_response(&mut stream, &response).await;
    });
    let client = HerdrClient::new(path);

    let result = client.ping().await;

    assert!(matches!(result, Err(ClientError::MismatchedId { .. })));
    server.await.unwrap();
}

#[tokio::test]
async fn retains_server_error_code_and_message() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        let mut response = fixture("error.json");
        response["id"] = request["id"].clone();
        write_response(&mut stream, &response).await;
    });
    let client = HerdrClient::new(path);

    let result = client.ping().await;

    assert!(matches!(
        result,
        Err(ClientError::Server { code, message })
            if code == "pane_not_found" && message.contains("w9:p9")
    ));
    server.await.unwrap();
}

#[tokio::test]
async fn rejects_an_unexpected_result_kind() {
    let (_directory, path, listener) = listener();
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let request = read_request(&mut stream).await;
        write_response(
            &mut stream,
            &json!({"id": request["id"], "result": {"type": "ok"}}),
        )
        .await;
    });
    let client = HerdrClient::new(path);

    let result = client.ping().await;

    assert!(matches!(
        result,
        Err(ClientError::UnexpectedResult {
            expected: "pong",
            ..
        })
    ));
    server.await.unwrap();
}
