use mockito::mock;
use rust_module::*;
use skyway_webrtc_gateway_api::data::{DataConnectionId, DataConnectionIdWrapper};
use skyway_webrtc_gateway_api::peer::{PeerCloseEvent, PeerConnectionEvent};
use std::sync::Mutex;

fn create_params() -> (PeerId, Token) {
    let peer_id = PeerId::new("hoge");
    let token = Token::try_create("pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
    (peer_id, token)
}

fn create_open_message(peer_id: &str, token: &str) -> String {
    format!(
        r#"{{
            "event": "OPEN",
            "params": {{
                "peer_id": "{}",
                "token": "{}"
            }}
        }}"#,
        peer_id, token
    )
}

fn create_connection_message(peer_id: &str, token: &str, data_connection_id: &str) -> String {
    format!(
        r#"{{
            "event": "CONNECTION",
            "params": {{
                "peer_id": "{}",
                "token": "{}"
            }},
            "data_params": {{
                "data_connection_id": "{}"
            }}
        }}"#,
        peer_id, token, data_connection_id
    )
}

fn create_close_message(peer_id: &str, token: &str) -> String {
    format!(
        r#"{{
            "event": "CLOSE",
            "params": {{
                "peer_id": "{}",
                "token": "{}"
            }}
        }}"#,
        peer_id, token
    )
}

#[tokio::test]
async fn test_create_peer() {
    let (message_tx, mut event_rx) = run().await;
    // set up parameters
    let (peer_id, token) = create_params();

    let counter = Mutex::new(-1isize);
    // GET /peers/{peer_id}/eventsに対応するmock
    // create peerの時点でOPENを返しているので、それ以外のイベントしか来ない
    let mock_event_api = {
        let open_message = create_open_message(peer_id.as_str(), token.as_str());
        let connection_message = create_connection_message(
            peer_id.as_str(),
            token.as_str(),
            "dc-102127d9-30de-413b-93f7-41a33e39d82b",
        );
        let close_message = create_close_message(peer_id.as_str(), token.as_str());

        let bind_url = format!(
            "/peers/{}/events?token={}",
            peer_id.as_str(),
            token.as_str()
        );

        mock("GET", bind_url.as_str())
            .with_status(reqwest::StatusCode::OK.as_u16() as usize)
            .with_header("content-type", "application/json")
            .with_body_from_fn(move |w| {
                let mut data = counter.lock().unwrap();
                *data += 1;
                // 最初はCreatePeerServiceのためにOPENを返す
                if *data == 0 {
                    w.write_all(open_message.clone().as_bytes())
                } else if *data == 1 {
                    // 次はCONNECTイベントを返す
                    w.write_all(connection_message.clone().as_bytes())
                } else {
                    // CLOSEイベントを返すとevent listenerが停止する
                    w.write_all(close_message.clone().as_bytes())
                }
            })
            .create()
    };

    // create peer
    let peer_info = {
        // POST /peersに対応するmock
        let mock_create_peer = {
            let ret_message = format!(
                r#"{{
                    "command_type": "PEERS_CREATE",
                    "params": {{
                        "peer_id": "{}",
                        "token": "{}"
                    }}
                }}"#,
                peer_id.as_str(),
                token.as_str()
            );

            mock("POST", "/peers")
                .with_status(reqwest::StatusCode::CREATED.as_u16() as usize)
                .with_header("content-type", "application/json")
                .with_body(ret_message)
                .create()
        };

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
            mockito::server_url(),
            peer_id.as_str()
        );

        // call create peer api
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();
        let _ = message_tx.send((tx, message)).await;
        let result = rx.await.unwrap();

        // serverが呼ばれたかチェックする
        mock_create_peer.assert();

        match serde_json::from_str::<ReturnMessage>(&result) {
            // PEER_CREATEが帰ってきていればpeer_infoを取り出す
            Ok(ReturnMessage::PEER_CREATE(message)) => {
                assert!(true);
                message.params
            }
            // それ以外のケースはバグが発生しているので、テストを失敗にする
            _ => {
                assert!(false);
                unreachable!();
            }
        }
    };

    // 期待値の生成
    // 1回目のevent listenerが取得するはずのCONNECT
    let expected_connect = ReturnMessage::PEER_EVENT(PeerEventMessage {
        result: true,
        command: "PEER_EVENT".into(),
        params: PeerEventEnum::CONNECTION(PeerConnectionEvent {
            params: peer_info.clone(),
            data_params: DataConnectionIdWrapper {
                data_connection_id: DataConnectionId::try_create(
                    "dc-102127d9-30de-413b-93f7-41a33e39d82b",
                )
                .unwrap(),
            },
        }),
    });
    // 2回目のevent listenerが取得するはずのCLOSE
    let expected_close = ReturnMessage::PEER_EVENT(PeerEventMessage {
        result: true,
        command: "PEER_EVENT".into(),
        params: PeerEventEnum::CLOSE(PeerCloseEvent {
            params: peer_info.clone(),
        }),
    });

    // 1つめのEVENTの取得
    let result = event_rx.recv().await.unwrap();
    assert_eq!(
        serde_json::from_str::<ReturnMessage>(&result).unwrap(),
        expected_connect
    );

    // 2つめのEVENTの取得
    let result = event_rx.recv().await.unwrap();
    assert_eq!(
        serde_json::from_str::<ReturnMessage>(&result).unwrap(),
        expected_close
    );

    // 3つめは来ない
}
