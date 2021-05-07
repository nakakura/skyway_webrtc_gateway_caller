use rust_module::*;

#[tokio::test]
async fn test_create_peer() {
    let (message_tx, _) = run().await;
    // set up parameters
    let peer_id = PeerId::new("hoge");

    // create peer
    let message = format!(
        r#"{{
                "command": "PEER_CREATE",
                "params": {{
                    "base_url": "{}",
                    "key": "api_key",
                    "domain": "localhost",
                    "peer_id": "{}",
                    "turn": true
                }}
            }}"#,
        "http://localhost:0",
        peer_id.as_str()
    );

    // call create peer api
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await.unwrap();

    match serde_json::from_str::<ReturnMessage>(&result) {
        // PEER_CREATEが帰ってきていればpeer_infoを取り出す
        Ok(ReturnMessage::ERROR(_)) => {
            assert!(true);
        }
        // それ以外のケースはバグが発生しているので、テストを失敗にする
        _ => {
            assert!(false);
            unreachable!();
        }
    }
}
