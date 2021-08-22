mod common;

use rust_module::prelude::*;
use rust_module::run;
use tokio::sync::mpsc;

use common::media;
use common::peer;
use common::terminal;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();

    // gatewayを操作するためのメッセージをやり取りするチャンネル
    let (message_tx, mut event_rx) = run("http://localhost:8000").await;

    // peer objectを作成
    let peer_info = peer::create_peer(&message_tx, api_key, "media_callee").await;

    // media socketの開放
    // video送信用ポート
    let media_socket_video = media::create_media(&message_tx, true).await;
    // audio送信用ポート
    let media_socket_audio = media::create_media(&message_tx, false).await;
    // rtcp送信用ポート
    let rtcp_socket = media::create_rtcp(&message_tx, true).await;

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
                ResponseMessageBodyEnum::Peer(PeerResponseMessageBodyEnum::Event(
                    PeerEventEnum::ERROR(error_event),
                )) => {
                    eprintln!("error recv: {:?}", error_event);
                }
                ResponseMessageBodyEnum::Peer(PeerResponseMessageBodyEnum::Event(
                    PeerEventEnum::CALL(call_event),
                )) => {
                    let media_connection_id = call_event.call_params.media_connection_id;
                    println!("{:?}", media_connection_id);
                    let answer_params = AnswerQuery {
                        constraints: Constraints {
                            video: true,
                            videoReceiveEnabled: Some(true),
                            audio: true,
                            audioReceiveEnabled: Some(true),
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
                        redirect_params: Some(RedirectParameters {
                            video: Some(
                                SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3000)
                                    .unwrap(),
                            ),
                            video_rtcp: Some(
                                SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3001)
                                    .unwrap(),
                            ),
                            audio: Some(
                                SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3002)
                                    .unwrap(),
                            ),
                            audio_rtcp: Some(
                                SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3003)
                                    .unwrap(),
                            ),
                        }),
                    };

                    println!("start answer {:?}", answer_params);
                    let result =
                        media::answer(&message_tx, media_connection_id, answer_params).await;
                    println!("result {:?}", result);
                }
                ResponseMessageBodyEnum::Peer(PeerResponseMessageBodyEnum::Event(
                    PeerEventEnum::CLOSE(close_event),
                )) => {
                    println!("{:?} has been deleted. \nExiting Program", close_event);
                    break;
                }
                ResponseMessageBodyEnum::Media(MediaResponseMessageBodyEnum::Event(event)) => {
                    println!("media event \n {:?}", event);
                }
                message => {
                    panic!("{:?}", message);
                }
            }
        }
    };

    tokio::join!(user_input_fut, event_fut);
}
