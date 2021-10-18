use tokio::sync::mpsc;

use response_message::*;
use rust_module::prelude::*;

use crate::common::ControlMessage;

#[allow(dead_code)]
pub async fn create_media(
    message_tx: &mpsc::Sender<ControlMessage>,
    is_video: bool,
) -> MediaSocket {
    let message = format!(
        r#"{{
            "type": "MEDIA",
            "command": "CONTENT_CREATE",
            "params": {{
                "is_video": {}
            }}
        }}"#,
        is_video
    );

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("craete data socket failed{:?}", result.err());
    }

    let response_message = ResponseMessage::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::ContentCreate(socket),
        ))) => socket,
        message => {
            panic!("craete data socket failed{:?}", message);
        }
    }
}

#[allow(dead_code)]
pub async fn create_rtcp(message_tx: &mpsc::Sender<ControlMessage>, is_video: bool) -> RtcpSocket {
    let message = format!(
        r#"{{
            "type": "MEDIA",
            "command": "RTCP_CREATE",
            "params": {}
        }}"#,
        is_video
    );

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("create rtcp failed{:?}", result.err());
    }

    let response_message = ResponseMessage::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::RtcpCreate(socket),
        ))) => socket,
        message => {
            panic!("create rtcp failed{:?}", message);
        }
    }
}

#[allow(dead_code)]
pub async fn call(
    message_tx: &mpsc::Sender<ControlMessage>,
    query: CallQuery,
) -> MediaConnectionId {
    let message = format!(
        r#"{{
        "type":"MEDIA",
        "command":"CALL",
        "params": {}
    }}"#,
        serde_json::to_string(&query).unwrap()
    );

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("call failed{:?}", result.err());
    }

    let response_message = ResponseMessage::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::Call(response),
        ))) => response.media_connection_id,
        message => {
            panic!("call failed{:?}", message);
        }
    }
}

#[allow(dead_code)]
pub async fn answer(
    message_tx: &mpsc::Sender<ControlMessage>,
    media_connection_id: MediaConnectionId,
    answer_query: AnswerQuery,
) -> AnswerResult {
    let message = format!(
        r#"{{
        "type":"MEDIA",
        "command":"ANSWER",
        "params":{{
            "media_connection_id": "{}",
            "answer_query": {}
        }}
    }}"#,
        media_connection_id.as_str(),
        serde_json::to_string(&answer_query).unwrap()
    );

    // create callback
    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
    let _ = message_tx.send((tx, message)).await;
    let result = rx.await;
    if result.is_err() {
        panic!("answer failed{:?}", result.err());
    }

    let response_message = ResponseMessage::from_str(&result.unwrap());

    match response_message {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::Answer(response),
        ))) => response,
        message => {
            panic!("answer failed{:?}", message);
        }
    }
}
