use std::sync::Mutex;

use mockito::mock;

use module::prelude::*;
use module::*;
use response_message::*;

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
    // サービスにメッセージを流し込むためのチャンネル
    // 操作は基本的にこのチャンネルで行う
    let (message_tx, mut event_rx) = run(&mockito::server_url()).await;
    // set up parameters
    let (peer_id, token) = create_params();

    // GET /peers/{peer_id}/eventsに対応するmock
    // create peerの時点でOPENを返しているので、それ以外のイベントしか来ない
    let mock_event_api = {
        // Mockの動作を変更するためのフラグとして利用する
        // このテストでは、OPEN, CONNECTION, CLOSEの順で返したいので、カウンタを利用する
        let counter = Mutex::new(-1isize);

        // OPENイベントとして返すメッセージ
        let open_message = create_open_message(peer_id.as_str(), token.as_str());
        // CONNECTIONイベントとして返すメッセージ
        let connection_message = create_connection_message(
            peer_id.as_str(),
            token.as_str(),
            "dc-102127d9-30de-413b-93f7-41a33e39d82b",
        );
        // CLOSEイベントとして返すメッセージ
        let close_message = create_close_message(peer_id.as_str(), token.as_str());

        // 参照) http://35.200.46.204/#/1.peers/peer_event
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

    // まず最初にPeerObjectを生成する
    let peer_info = {
        // POST /peersに対応するmock
        // http://35.200.46.204/#/1.peers/peer
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

        // POST /peersで送信するbody
        // http://35.200.46.204/#/1.peers/peer
        let body = format!(
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
        let _ = message_tx.send((tx, body)).await;
        let result = rx.await;
        if result.is_err() {
            assert!(false);
            unreachable!();
        }

        let result = ResponseMessage::from_str(&result.unwrap());

        // serverが呼ばれたかチェックする
        mock_create_peer.assert();

        match result {
            // PeerCreateが帰ってきていればpeer_infoを取り出す
            Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
                PeerResponseMessageBodyEnum::Create(peer_info),
            ))) => {
                assert!(true);
                peer_info
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
    let expected_connect =
        PeerResponseMessageBodyEnum::Event(PeerEventEnum::CONNECTION(PeerConnectionEvent {
            params: peer_info.clone(),
            data_params: DataConnectionIdWrapper {
                data_connection_id: DataConnectionId::try_create(
                    "dc-102127d9-30de-413b-93f7-41a33e39d82b",
                )
                .unwrap(),
            },
        }))
        .create_response_message();

    // 2回目のevent listenerが取得するはずのCLOSE
    let expected_close = PeerResponseMessageBodyEnum::Event(PeerEventEnum::CLOSE(PeerCloseEvent {
        params: peer_info.clone(),
    }))
    .create_response_message();
    println!("{:?}", expected_close);

    // serverが呼ばれたかチェックする
    mock_event_api.assert();

    // 1つめのEVENTの取得
    let result = event_rx.recv().await.unwrap();
    assert_eq!(result, serde_json::to_string(&expected_connect).unwrap());

    // 2つめのEVENTの取得
    let result = event_rx.recv().await.unwrap();
    assert_eq!(result, serde_json::to_string(&expected_close).unwrap());

    // 3つめは来ない
}
