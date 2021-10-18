use rust_module::*;

#[tokio::test]
async fn test_create_peer() {
    let (message_tx, _) = run("http://localhost:0").await;
    // set up parameters
    let peer_id = PeerId::new("hoge");

    // create peer
    let message = format!(
        r#"{{
                "type": "PEER",
                "command": "CREATE",
                "params": {{
                    "key": "api_key",
                    "domain": "localhost",
                    "peer_id": "{}",
                    "turn": true
                }}
            }}"#,
        peer_id.as_str()
    );

    // call create peer api
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        assert!(false);
        unreachable!();
    }

    let result = ResponseMessage::from_str(&result.unwrap());

    match result {
        // Errorが帰ってくるはず
        Ok(ResponseMessage::Error(_)) => {
            assert!(true);
        }
        // それ以外のケースはバグが発生しているので、テストを失敗にする
        _ => {
            assert!(false);
            unreachable!();
        }
    }
}
