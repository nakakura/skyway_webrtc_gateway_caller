use serde::Serialize;
use tokio::sync::mpsc;

use skyway_webrtc_gateway_caller::prelude::common::*;
use skyway_webrtc_gateway_caller::prelude::data::*;
use skyway_webrtc_gateway_caller::prelude::response_parser::*;

use super::ControlMessage;

#[allow(dead_code)]
pub async fn create_data(message_tx: &mpsc::Sender<ControlMessage>) -> SocketInfo<DataId> {
    let message = r#"{
        "type": "DATA",
        "command": "CREATE",
        "params": ""
    }"#
    .to_string();
    // 処理を開始

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("data socket open failed{:?}", result.err());
    }

    let response_message = ResponseResult::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseResult::Success(ResponseMessage::Data(DataResponse::Create(socket)))) => socket,
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

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("data socket close failed{:?}", result.err());
    }

    let response_message = ResponseResult::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseResult::Success(ResponseMessage::Data(DataResponse::Delete(
            DataIdWrapper { data_id },
        )))) => data_id,
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

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("connect failed{:?}", result.err());
    }

    let response_message = ResponseResult::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseResult::Success(ResponseMessage::Data(DataResponse::Connect(
            connection_id_wrapper,
        )))) => connection_id_wrapper.data_connection_id,
        _ => {
            panic!("connect failed")
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

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("data redirect failed{:?}", result.err());
    }

    let response_message = ResponseResult::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseResult::Success(ResponseMessage::Data(DataResponse::Redirect(
            connection_id_wrapper,
        )))) => connection_id_wrapper.data_connection_id,
        _ => {
            panic!("data redirect failed")
        }
    }
}
