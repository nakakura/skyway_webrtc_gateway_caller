use mockito::mock;
use rust_module::prelude::*;
use rust_module::*;

fn create_data_message() -> String {
    r#"{
        "type": "DATA",
        "command": "CREATE",
        "params": ""
    }"#
    .to_string()
}

fn delete_data_message(data_id: &str) -> String {
    format!(
        r#"{{
            "type": "DATA",
            "command": "DELETE",
            "params": {{
                "data_id": "{}"
            }}
        }}"#,
        data_id
    )
}

#[tokio::test]
async fn test_create_data() {
    // create data apiに対応するMock
    // socket割当に成功したケースとして値を返す
    // http://35.200.46.204/#/2.data/data
    let _mock_create_data_api = mock("POST", "/data")
        .with_status(reqwest::StatusCode::CREATED.as_u16() as usize)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "data_id": "da-50a32bab-b3d9-4913-8e20-f79c90a6a211",
                "port": 10001,
                "ip_v4": "127.0.0.1"
            }"#,
        )
        .create();

    // サービスにメッセージを流し込むためのチャンネル
    // 操作は基本的にこのチャンネルで行う
    let (message_tx, _event_rx) = run(&mockito::server_url()).await;

    // 操作の結果を受け取るためのチャンネル
    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    // 操作指示を生成
    let message = create_data_message();
    let body = serde_json::from_str::<ServiceParams>(&message);

    // 処理を開始
    let result = message_tx.send((tx, body.unwrap())).await;
    assert!(result.is_ok());
    let result = rx.await.unwrap();

    // evaluate
    assert_eq!(serde_json::to_string(&result).unwrap(),
               "{\"is_success\":true,\"result\":{\"type\":\"DATA\",\"command\":\"CREATE\",\"data_id\":\"da-50a32bab-b3d9-4913-8e20-f79c90a6a211\",\"ip_v4\":\"127.0.0.1\",\"port\":10001}}");
}

#[tokio::test]
async fn test_delete_data() {
    // delete data apiに対応するMock
    // socket開放に成功したケースとして値を返す
    // http://35.200.46.204/#/2.data/data_delete
    let data_id = "da-50a32bab-b3d9-4913-8e20-f79c90a6a211";
    let url = format!("/data/{}", data_id);
    let _mock_create_data_api = mock("DELETE", url.as_str())
        .with_status(reqwest::StatusCode::NO_CONTENT.as_u16() as usize)
        .with_header("content-type", "application/json")
        .create();

    // サービスにメッセージを流し込むためのチャンネル
    // 操作は基本的にこのチャンネルで行う
    let (message_tx, _event_rx) = run(&mockito::server_url()).await;

    // 操作の結果を受け取るためのチャンネル
    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    // 操作指示を生成
    let message = delete_data_message(data_id);
    let body = serde_json::from_str::<ServiceParams>(&message);

    // 処理を開始
    let result = message_tx.send((tx, body.unwrap())).await;
    assert!(result.is_ok());
    let result = rx.await.unwrap();

    // evaluate
    assert_eq!(
        serde_json::to_string(&result).unwrap(),
        "{\"is_success\":true,\"result\":{\"type\":\"DATA\",\"command\":\"DELETE\",\"data_id\":\"da-50a32bab-b3d9-4913-8e20-f79c90a6a211\"}}"
    );
}
