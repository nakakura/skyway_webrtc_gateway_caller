use mockito::mock;
use rust_module::*;

fn create_params() -> (PeerId, Token) {
    let peer_id = PeerId::new("hoge");
    let token = Token::try_create("pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
    (peer_id, token)
}

#[tokio::test]
async fn test_create_peer() {
    // set up parameters
    let (peer_id, token) = create_params();

    // create response json
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

    // set up server mock
    let mock_create_peer_api = mock("POST", "/peers")
        .with_status(reqwest::StatusCode::CREATED.as_u16() as usize)
        .with_header("content-type", "application/json")
        .with_body(ret_message)
        .create();

    let ret_message = format!(
        r#"{{
            "event": "OPEN",
            "params": {{
                "peer_id": "{}",
                "token": "{}"
            }}
        }}"#,
        peer_id.as_str(),
        token.as_str()
    );

    let url = format!(
        "/peers/{}/events?token={}",
        peer_id.as_str(),
        token.as_str()
    );
    let mock_event_api = mock("GET", url.as_str())
        .with_status(reqwest::StatusCode::OK.as_u16() as usize)
        .with_header("content-type", "application/json")
        .with_body(ret_message)
        .create();

    let (message_tx, _event_rx) = run().await;
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();

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

    let _ = message_tx.send((tx, message)).await;
    let result = rx.await.unwrap();

    if let Ok(ReturnMessage::PEER_CREATE(message)) = serde_json::from_str::<ReturnMessage>(&result)
    {
        assert_eq!(message.params, PeerInfo::new(peer_id, token));
    } else {
        assert!(false);
    }

    // server called
    mock_create_peer_api.assert();
    mock_event_api.assert();
}
