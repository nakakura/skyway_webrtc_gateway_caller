mod common;

use rust_module::prelude::*;
use rust_module::run;
use tokio::sync::mpsc;

use common::peer;
use common::terminal;
use common::ControlMessage;

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

pub async fn answer(
    message_tx: &mpsc::Sender<ControlMessage>,
    media_connection_id: MediaConnectionId,
    answer_query: AnswerQuery,
) -> AnswerResponseParams {
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

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();

    // gatewayを操作するためのメッセージをやり取りするチャンネル
    let (message_tx, mut event_rx) = run("http://localhost:8000").await;

    // peer objectを作成
    let peer_info = peer::create_peer(&message_tx, api_key, "media_callee").await;

    // media socketの開放
    // video送信用ポート
    let media_socket_video = create_media(&message_tx, true).await;
    // audio送信用ポート
    let media_socket_audio = create_media(&message_tx, false).await;
    // rtcp送信用ポート
    let rtcp_socket = create_rtcp(&message_tx, true).await;

    // terminalの読み込み
    let (terminal_tx, mut terminal_rx) = mpsc::channel::<String>(10);
    tokio::spawn(terminal::read_stdin(terminal_tx));

    // terminalからコマンドを受け取り処理を実施する
    // exitコマンドのみ
    let user_input_fut = async {
        while let Some(message) = terminal_rx.recv().await {
            match message.as_str() {
                "exit" => {
                    peer::delete_peer(&message_tx, &peer_info).await;
                    break;
                }
                _ => {}
            }
        }
    };

    // eventを出力する
    let event_fut = async {
        while let Some(ResponseMessage::Success(event)) = event_rx.recv().await {
            match event {
                ResponseMessageBodyEnum::PeerEvent(PeerEventEnum::ERROR(error_event)) => {
                    eprintln!("error recv: {:?}", error_event);
                }
                ResponseMessageBodyEnum::PeerEvent(PeerEventEnum::CALL(call_event)) => {
                    let media_connection_id = call_event.call_params.media_connection_id;
                    let answer_params = AnswerQuery {
                        constraints: Constraints {
                            video: false,
                            videoReceiveEnabled: None,
                            audio: false,
                            audioReceiveEnabled: None,
                            video_params: Some(MediaParams {
                                band_width: 1500,
                                codec: "H264".to_string(),
                                media_id: media_socket_video.get_id().unwrap(),
                                rtcp_id: rtcp_socket.get_id(),
                                payload_type: None,
                                sampling_rate: None,
                            }),
                            audio_params: Some(MediaParams {
                                band_width: 1500,
                                codec: "OPUS".to_string(),
                                media_id: media_socket_audio.get_id().unwrap(),
                                rtcp_id: None,
                                payload_type: None,
                                sampling_rate: None,
                            }),
                            metadata: None,
                        },
                        redirect_params: None,
                    };

                    let result = answer(&message_tx, media_connection_id, answer_params).await;
                    println!("result {:?}", result);
                }
                ResponseMessageBodyEnum::PeerEvent(PeerEventEnum::CLOSE(close_event)) => {
                    println!("{:?} has been deleted. \nExiting Program", close_event);
                    break;
                }
                message => {
                    panic!("{:?}", message);
                }
            }
        }
    };

    tokio::join!(user_input_fut, event_fut);
}
