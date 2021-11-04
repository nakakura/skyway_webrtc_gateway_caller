use tokio::sync::mpsc;

use response_message::*;
use module::prelude::*;

use super::ControlMessage;

// SkyWayはCreate Peerしたあとサーバ側で承認が出るまで待たないと、PeerObjectの生成が成功したかどうかわからない
// WebRTC Gatewayの処理としては、Peer Eventとして結果が受け取れるようになっているが、
// PEER_CREATE APIの処理中にこのイベントのチェックまでやっているので、ユーザはイベントの監視をする必要がない
#[allow(dead_code)]
pub async fn create_peer(
    message_tx: &mpsc::Sender<ControlMessage>,
    api_key: String,
    peer_id: &str,
) -> PeerInfo {
    // create control message
    let message = format!(
        r#"{{
                "type": "PEER",
                "command": "CREATE",
                "params": {{
                    "key": "{}",
                    "domain": "localhost",
                    "peer_id": "{}",
                    "turn": true
                }}
            }}"#,
        api_key, peer_id,
    );

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("peer create failed{:?}", result.err());
    }

    let response_message = ResponseMessage::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Create(result),
        ))) => result,
        Ok(ResponseMessage::Error(e)) => {
            panic!("{:?}", e);
        }
        _ => {
            unreachable!()
        }
    }
}

#[allow(dead_code)]
pub async fn delete_peer(message_tx: &mpsc::Sender<ControlMessage>, peer_info: &PeerInfo) {
    // create control message
    let message = format!(
        r#"{{
                "type": "PEER",
                "command": "DELETE",
                "params": {{
                    "peer_id": "{}",
                    "token": "{}"
                }}
            }}"#,
        peer_info.peer_id().as_str(),
        peer_info.token().as_str()
    );

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("peer delete failed{:?}", result.err());
    }

    let response_message = ResponseMessage::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Delete(result),
        ))) => {
            println!("Peer {:?} is deleted", result);
        }
        Ok(ResponseMessage::Error(e)) => {
            panic!("{:?}", e);
        }
        _ => {
            unreachable!()
        }
    }
}
