use rust_module::prelude::*;
use serde::Serialize;
use tokio::sync::mpsc;

use super::ControlMessage;

#[allow(dead_code)]
pub async fn create_data(message_tx: &mpsc::Sender<ControlMessage>) -> DataSocket {
    let body_json = r#"{
        "type": "DATA",
        "command": "CREATE",
        "params": ""
    }"#
    .to_string();
    let body = serde_json::from_str::<ServiceParams>(&body_json);
    // 処理を開始

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Data(
            DataResponseMessageBodyEnum::Create(socket),
        ))) => socket,
        _ => {
            panic!("data socket open failed")
        }
    }
}

#[allow(dead_code)]
pub async fn delete_data(message_tx: &mpsc::Sender<ControlMessage>, data_id: DataId) -> DataId {
    let body_json = format!(
        r#"{{
            "type": "DATA",
            "command": "DELETE",
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
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Data(
            DataResponseMessageBodyEnum::Delete(DataIdWrapper { data_id }),
        ))) => data_id,
        _ => {
            panic!("data socket close failed")
        }
    }
}

#[allow(dead_code)]
pub async fn connect(
    message_tx: &mpsc::Sender<ControlMessage>,
    query: ConnectQuery,
) -> DataConnectionId {
    // create message body
    #[derive(Serialize)]
    struct ConnectMessage {
        r#type: String,
        command: String,
        params: ConnectQuery,
    }
    let message = ConnectMessage {
        r#type: "DATA".into(),
        command: "CONNECT".into(),
        params: query,
    };
    let body_json = serde_json::to_value(&message).unwrap();
    let body = serde_json::from_value::<ServiceParams>(body_json).unwrap();

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body)).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Data(
            DataResponseMessageBodyEnum::Connect(connection_id_wrapper),
        ))) => connection_id_wrapper.data_connection_id,
        _ => {
            panic!("data socket close failed")
        }
    }
}

#[derive(Serialize)]
pub struct RedirectParams {
    pub data_connection_id: DataConnectionId,
    pub feed_params: Option<DataIdWrapper>,
    pub redirect_params: Option<SocketInfo<PhantomId>>,
}

#[allow(dead_code)]
pub async fn redirect(
    message_tx: &mpsc::Sender<ControlMessage>,
    params: RedirectParams,
) -> DataConnectionId {
    #[derive(Serialize)]
    struct Message {
        r#type: String,
        command: String,
        params: RedirectParams,
    }
    let message = Message {
        r#type: "DATA".into(),
        command: "REDIRECT".into(),
        params: params,
    };

    let body_json = serde_json::to_value(&message).unwrap();
    let body = serde_json::from_value::<ServiceParams>(body_json).unwrap();
    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body)).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Data(
            DataResponseMessageBodyEnum::Redirect(connection_id_wrapper),
        ))) => connection_id_wrapper.data_connection_id,
        _ => {
            panic!("data redirect failed")
        }
    }
}
