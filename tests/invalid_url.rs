use std::sync::Mutex;

use mockito::mock;
use rust_module::*;
use skyway_webrtc_gateway_api::data::{DataConnectionId, DataConnectionIdWrapper};
use skyway_webrtc_gateway_api::peer::{PeerCloseEvent, PeerConnectionEvent};

fn create_params() -> (PeerId, Token) {
    let peer_id = PeerId::new("hoge");
    let token = Token::try_create("pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
    (peer_id, token)
}

#[tokio::test]
async fn test_create_peer() {
    let (message_tx, mut event_rx) = run().await;
    // set up parameters
    let (peer_id, token) = create_params();

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
        Ok(ReturnMessage::ERROR(message)) => {
            assert!(true);
        }
        // それ以外のケースはバグが発生しているので、テストを失敗にする
        _ => {
            assert!(false);
            unreachable!();
        }
    }
}
