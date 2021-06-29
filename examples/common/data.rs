use rust_module::prelude::*;
use serde::Serialize;
use tokio::sync::mpsc;

use super::ControlMessage;

pub(crate) async fn create_data(message_tx: &mpsc::Sender<ControlMessage>) -> SocketInfo<DataId> {
    let body_json = r#"{
        "command": "DATA_CREATE",
        "params": ""
    }"#
    .to_string();
    let body = serde_json::from_str::<ServiceParams>(&body_json);
    // 処理を開始

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::DataCreate(socket))) => socket,
        _ => {
            panic!("data socket open failed")
        }
    }
}

pub(crate) async fn delete_data(
    message_tx: &mpsc::Sender<ControlMessage>,
    data_id: DataId,
) -> DataId {
    let body_json = format!(
        r#"{{
            "command": "DATA_DELETE",
            "params": {{
                "data_id": "{}"
            }}
        }}"#,
        data_id.as_str()
    );
    let body = serde_json::from_str::<ServiceParams>(&body_json);

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::DataDelete(data_id))) => data_id,
        _ => {
            panic!("data socket close failed")
        }
    }
}

pub(crate) async fn connect(
    message_tx: &mpsc::Sender<ControlMessage>,
    peer_info: &PeerInfo,
    data_id: DataId,
) {
    // create parameter
    let target_id = PeerId::new("target_id");
    let data_id = DataIdWrapper { data_id: data_id };
    let query = ConnectQuery {
        peer_id: peer_info.peer_id(),
        token: peer_info.token(),
        options: None,
        target_id: target_id,
        params: Some(data_id.clone()),
        redirect_params: None,
    };

    // create message body
    #[derive(Serialize)]
    struct ConnectMessage {
        command: String,
        params: ConnectQuery,
    }
    let message = ConnectMessage {
        command: "DATA_CONNECT".into(),
        params: query,
    };
    let body_json = serde_json::to_value(&message).unwrap();
    let body = serde_json::from_value::<ServiceParams>(body_json).unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body)).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::DataConnect(
            connection_id_wrapper,
        ))) => {
            println!("{:?}", connection_id_wrapper);
            data_id;
        }
        _ => {
            panic!("data socket close failed")
        }
    }
}
