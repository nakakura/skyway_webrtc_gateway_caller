use mockito::mock;
use rust_module::*;

fn create_data_message() -> String {
    r#"{
        "command": "DATA_CREATE",
        "params": ""
    }"#
    .to_string()
}

#[tokio::test]
async fn test_create_data() {
    // create data apiに対応するMock
    // socket開放に成功したケースとして値を返す
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
    let (tx, rx) = tokio::sync::oneshot::channel::<ReturnMessage>();
    // 操作指示を生成
    let message = create_data_message();
    let body = serde_json::from_str::<ServiceParams>(&message);

    // 処理を開始
    let result = message_tx.send((tx, body.unwrap())).await;
    assert!(result.is_ok());
    let result = rx.await.unwrap();

    // evaluate
    assert_eq!(serde_json::to_string(&result).unwrap(), "{\"result\":true,\"command\":\"DATA_CREATE\",\"params\":{\"data_id\":\"da-50a32bab-b3d9-4913-8e20-f79c90a6a211\",\"ip_v4\":\"127.0.0.1\",\"port\":10001}}");
}
