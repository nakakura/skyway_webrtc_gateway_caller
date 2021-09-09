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
    use serde::Serialize;

    #[derive(Serialize)]
    struct Parameter {
        r#type: String,
        command: String,
        params: CallQuery,
    }

    let paramter = Parameter {
        r#type: "MEDIA".into(),
        command: "CALL".into(),
        params: query,
    };

    let json_message = serde_json::to_string(&paramter).unwrap();
    let body = serde_json::from_str::<request_message::ServiceParams>(&json_message);

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
    use serde::Serialize;

    #[derive(Serialize)]
    struct AnswerParameter {
        r#type: String,
        command: String,
        params: InternalParams,
    }

    #[derive(Serialize)]
    struct InternalParams {
        media_connection_id: MediaConnectionId,
        answer_query: AnswerQuery,
    }

    let param = AnswerParameter {
        r#type: "MEDIA".into(),
        command: "ANSWER".into(),
        params: InternalParams {
            media_connection_id,
            answer_query,
        },
    };
    let json_message = serde_json::to_string(&param).unwrap();
    let body = serde_json::from_str::<request_message::ServiceParams>(&json_message);

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
