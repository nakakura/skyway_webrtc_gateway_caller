use rust_module::prelude::*;
use tokio::sync::mpsc;

use crate::common::ControlMessage;

#[allow(dead_code)]
pub async fn create_media(
    message_tx: &mpsc::Sender<ControlMessage>,
    is_video: bool,
) -> SocketInfo<MediaId> {
    let body_json = format!(
        r#"{{
            "command": "MEDIA_CONTENT_CREATE",
            "params": {}
        }}"#,
        is_video
    );
    let body = serde_json::from_str::<ServiceParams>(&body_json);
    // 処理を開始

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::MediaContentCreate(socket))) => socket,
        message => {
            panic!("data socket open failed{:?}", message);
        }
    }
}

#[allow(dead_code)]
pub async fn create_rtcp(
    message_tx: &mpsc::Sender<ControlMessage>,
    is_video: bool,
) -> SocketInfo<RtcpId> {
    let body_json = format!(
        r#"{{
            "command": "MEDIA_RTCP_CREATE",
            "params": {}
        }}"#,
        is_video
    );
    let body = serde_json::from_str::<ServiceParams>(&body_json);
    // 処理を開始

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::MediaRtcpCreate(socket))) => socket,
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
        command: String,
        params: InternalParams,
    }

    #[derive(Serialize)]
    struct InternalParams {
        media_connection_id: MediaConnectionId,
        answer_query: AnswerQuery,
    }

    let param = AnswerParameter {
        command: "MEDIA_ANSWER".into(),
        params: InternalParams {
            media_connection_id,
            answer_query,
        },
    };
    let json_message = serde_json::to_string(&param).unwrap();
    let body = serde_json::from_str::<ServiceParams>(&json_message);

    let (tx, rx) = tokio::sync::oneshot::channel::<ResponseMessage>();
    let _ = message_tx.send((tx, body.unwrap())).await;
    match rx.await {
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::MediaAnswer(response))) => response,
        message => {
            panic!("data socket open failed{:?}", message);
        }
    }
}
