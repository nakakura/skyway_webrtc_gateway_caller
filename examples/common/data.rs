use serde::Serialize;
use tokio::sync::mpsc;

use response_message::*;
use rust_module::prelude::*;

use super::ControlMessage;

#[allow(dead_code)]
pub async fn create_data(message_tx: &mpsc::Sender<ControlMessage>) -> DataSocket {
    let message = r#"{
        "type": "DATA",
        "command": "CREATE",
        "params": ""
    }"#
    .to_string();
    // 処理を開始

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, message)).await;
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
    let message = format!(
        r#"{{
            "type": "DATA",
            "command": "DELETE",
            "params": {{
                "data_id": "{}"
            }}
        }}"#,
        data_id.as_str()
    );

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, message)).await;
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
    let message = format!(
        r#"{{
            "type":"DATA",
            "command":"CONNECT",
            "params":{}
        }}"#,
        serde_json::to_string(&query).unwrap()
    );

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, message)).await;
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
    let message = format!(
        r#"{{
        "type":"DATA",
        "command":"REDIRECT",
        "params": {}
    }}"#,
        serde_json::to_string(&params).unwrap()
    );

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, message)).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Data(
            DataResponseMessageBodyEnum::Redirect(connection_id_wrapper),
        ))) => connection_id_wrapper.data_connection_id,
        _ => {
            panic!("data redirect failed")
        }
    }
}
