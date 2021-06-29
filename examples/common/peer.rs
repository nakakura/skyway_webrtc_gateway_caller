use rust_module::prelude::*;
use tokio::sync::mpsc;

use super::ControlMessage;

// SkyWayはCreate Peerしたあとサーバ側で承認が出るまで待たないと、PeerObjectの生成が成功したかどうかわからない
// WebRTC Gatewayの処理としては、Peer Eventとして結果が受け取れるようになっているが、
// PEER_CREATE APIの処理中にこのイベントのチェックまでやっているので、ユーザはイベントの監視をする必要がない
pub(crate) async fn create_peer(
    message_tx: &mpsc::Sender<ControlMessage>,
    api_key: String,
    peer_id: &str,
) -> PeerInfo {
    // create control message
    let message = format!(
        r#"{{
                "command": "PEER_CREATE",
                "params": {{
                    "key": "{}",
                    "domain": "localhost",
                    "peer_id": "{}",
                    "turn": true
                }}
            }}"#,
        api_key, peer_id,
    );
    let message = serde_json::from_str::<ServiceParams>(&message).unwrap();

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();

    let _ = message_tx.send((tx, message)).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::PeerCreate(result))) => result,
        Ok(ResponseMessage::Error(e)) => {
            panic!("{:?}", e);
        }
        _ => {
            unreachable!()
        }
    }
}

pub(crate) async fn delete_peer(message_tx: &mpsc::Sender<ControlMessage>, peer_info: &PeerInfo) {
    println!("start deleting {:?}", peer_info);
    // create control message
    let message = format!(
        r#"{{
                "command": "PEER_DELETE",
                "params": {{
                    "peer_id": "{}",
                    "token": "{}"
                }}
            }}"#,
        peer_info.peer_id().as_str(),
        peer_info.token().as_str()
    );
    let message = serde_json::from_str::<ServiceParams>(&message).unwrap();

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();

    let _ = message_tx.send((tx, message)).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::PeerDelete(result))) => {
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
