use herdr_webmaster::herdr::framing::{
    FramingError, read_json_line, read_optional_json_line, write_json_line,
};
use serde_json::{Value, json};
use tokio::io::{AsyncWriteExt, BufReader, duplex};

#[tokio::test]
async fn reads_a_message_split_across_writes() {
    let (mut writer, reader) = duplex(128);
    let write_task = tokio::spawn(async move {
        writer.write_all(br#"{"event":"pane."#).await.unwrap();
        writer
            .write_all(b"agent_status_changed\",\"data\":{}}\n")
            .await
            .unwrap();
    });
    let mut reader = BufReader::new(reader);

    let value: Value = read_json_line(&mut reader)
        .await
        .expect("complete JSON line");

    write_task.await.unwrap();
    assert_eq!(value["event"], "pane.agent_status_changed");
}

#[tokio::test]
async fn reads_several_messages_from_one_write() {
    let (mut writer, reader) = duplex(128);
    writer.write_all(b"{\"id\":1}\n{\"id\":2}\n").await.unwrap();
    let mut reader = BufReader::new(reader);

    let first: Value = read_json_line(&mut reader).await.unwrap();
    let second: Value = read_json_line(&mut reader).await.unwrap();

    assert_eq!(first["id"], 1);
    assert_eq!(second["id"], 2);
}

#[tokio::test]
async fn writes_one_flushed_json_line() {
    let (mut client, mut server) = duplex(128);

    write_json_line(&mut client, &json!({"id": "req-1"}))
        .await
        .unwrap();

    let mut bytes = vec![0; 15];
    let read = tokio::io::AsyncReadExt::read(&mut server, &mut bytes)
        .await
        .unwrap();
    assert_eq!(&bytes[..read], b"{\"id\":\"req-1\"}\n");
}

#[tokio::test]
async fn blank_line_is_an_error() {
    let (mut writer, reader) = duplex(16);
    writer.write_all(b"\n").await.unwrap();
    let mut reader = BufReader::new(reader);

    let result = read_json_line::<_, Value>(&mut reader).await;

    assert!(matches!(result, Err(FramingError::EmptyLine)));
}

#[tokio::test]
async fn invalid_json_is_an_error() {
    let (mut writer, reader) = duplex(16);
    writer.write_all(b"{nope}\n").await.unwrap();
    let mut reader = BufReader::new(reader);

    let result = read_json_line::<_, Value>(&mut reader).await;

    assert!(matches!(result, Err(FramingError::Json(_))));
}

#[tokio::test]
async fn optional_reader_returns_none_at_eof() {
    let (writer, reader) = duplex(16);
    drop(writer);
    let mut reader = BufReader::new(reader);

    let result = read_optional_json_line::<_, Value>(&mut reader)
        .await
        .expect("clean EOF");

    assert!(result.is_none());
}

#[tokio::test]
async fn required_reader_reports_eof() {
    let (writer, reader) = duplex(16);
    drop(writer);
    let mut reader = BufReader::new(reader);

    let result = read_json_line::<_, Value>(&mut reader).await;

    assert!(matches!(result, Err(FramingError::Eof)));
}
