use tokio::sync::mpsc;

use response_message::*;
use rust_module::prelude::*;

use crate::common::ControlMessage;

#[allow(dead_code)]
pub async fn create_media(
    message_tx: &mpsc::Sender<ControlMessage>,
    is_video: bool,
) -> MediaSocket {
    let body_json = format!(
        r#"{{
            "type": "MEDIA",
            "command": "CONTENT_CREATE",
            "params": {{
                "is_video": {}
            }}
        }}"#,
        is_video
    );
    let body = serde_json::from_str::<request_message::ServiceParams>(&body_json);
    // 処理を開始
    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::ContentCreate(socket),
        ))) => socket,
        message => {
            panic!("media socket open failed{:?}", message);
        }
    }
}

#[allow(dead_code)]
pub async fn create_rtcp(message_tx: &mpsc::Sender<ControlMessage>, is_video: bool) -> RtcpSocket {
    let body_json = format!(
        r#"{{
            "type": "MEDIA",
            "command": "RTCP_CREATE",
            "params": {}
        }}"#,
        is_video
    );
    let body = serde_json::from_str::<request_message::ServiceParams>(&body_json);
    // 処理を開始

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::RtcpCreate(socket),
        ))) => socket,
        message => {
            panic!("data socket open failed{:?}", message);
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

    let body = serde_json::from_str::<request_message::ServiceParams>(&message);

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::Call(response),
        ))) => response.media_connection_id,
        message => {
            panic!("data socket open failed{:?}", message);
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
    let body = serde_json::from_str::<request_message::ServiceParams>(&message);

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::Answer(response),
        ))) => response,
        message => {
            panic!("data socket open failed{:?}", message);
        }
    }
}
