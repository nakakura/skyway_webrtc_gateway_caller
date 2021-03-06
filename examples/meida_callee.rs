use tokio::sync::mpsc;

use common::media;
use common::peer;
use common::terminal;
use skyway_webrtc_gateway_caller::prelude::common::*;
use skyway_webrtc_gateway_caller::prelude::media::*;
use skyway_webrtc_gateway_caller::prelude::peer::PeerEventEnum;
use skyway_webrtc_gateway_caller::prelude::response_parser::{
    MediaResponse, PeerResponse, ResponseMessage, ResponseResult,
};
use skyway_webrtc_gateway_caller::run;

mod common;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();

    // gatewayを操作するためのメッセージをやり取りするチャンネル
    let (message_tx, mut event_rx) = run("http://localhost:8000").await;

    // peer objectを作成
    let peer_info: PeerInfo = peer::create_peer(&message_tx, api_key, "media_callee").await;

    // media socketの開放
    // video送信用ポート
    let media_socket_video: SocketInfo<MediaId> = media::create_media(&message_tx, true).await;
    // audio送信用ポート
    let media_socket_audio: SocketInfo<MediaId> = media::create_media(&message_tx, false).await;
    // rtcp送信用ポート
    let rtcp_socket: SocketInfo<RtcpId> = media::create_rtcp(&message_tx, true).await;

    // 受信用ポート
    let video_recv_sock = SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3000).unwrap();
    let video_rtcp_recv_sock =
        SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3001).unwrap();
    let audio_recv_sock = SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3002).unwrap();
    let audio_rtcp_recv_sock =
        SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 3003).unwrap();

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
        println!("waiting connection from a neighbour");
        while let Some(message) = event_rx.recv().await {
            if let ResponseResult::Success(event) = ResponseResult::from_str(&message).unwrap() {
                match event {
                    ResponseMessage::Peer(PeerResponse::Event(PeerEventEnum::ERROR(
                        error_event,
                    ))) => {
                        eprintln!("error recv: {:?}", error_event);
                    }
                    ResponseMessage::Peer(PeerResponse::Event(PeerEventEnum::CALL(call_event))) => {
                        let media_connection_id = call_event.call_params.media_connection_id;
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
                                video: Some(video_recv_sock.clone()),
                                video_rtcp: Some(video_rtcp_recv_sock.clone()),
                                audio: Some(audio_recv_sock.clone()),
                                audio_rtcp: Some(audio_rtcp_recv_sock.clone()),
                            }),
                        };

                        let _result =
                            media::answer(&message_tx, media_connection_id, answer_params).await;
                    }
                    ResponseMessage::Peer(PeerResponse::Event(PeerEventEnum::CLOSE(
                        close_event,
                    ))) => {
                        println!("{:?} has been deleted. \nExiting Program", close_event);
                        break;
                    }
                    ResponseMessage::Media(MediaResponse::Event(event)) => {
                        println!("media event \n {:?}", event);
                        match event {
                            MediaConnectionEventEnum::READY(_) => {
                                // send info
                                println!(
                                    "you can send video to: {}:{}",
                                    media_socket_video.ip(),
                                    media_socket_video.port()
                                );
                                println!(
                                    "you can send video rtcp to: {}:{}",
                                    rtcp_socket.ip(),
                                    rtcp_socket.port()
                                );
                                println!(
                                    "you can send audio to: {}:{}",
                                    media_socket_audio.ip(),
                                    media_socket_audio.port()
                                );
                                println!("you don't set audio rtcp forwarding config");

                                // redirect info
                                println!(
                                    "The received video will be transferred to {}:{}",
                                    video_recv_sock.ip(),
                                    video_recv_sock.port()
                                );
                                println!(
                                    "The received video rtcp will be transferred to {}:{}",
                                    video_rtcp_recv_sock.ip(),
                                    video_rtcp_recv_sock.port()
                                );
                                println!(
                                    "The received audio will be transferred to {}:{}",
                                    audio_recv_sock.ip(),
                                    audio_recv_sock.port()
                                );
                                println!(
                                    "The received audio rtcp will be transferred to {}:{}",
                                    audio_rtcp_recv_sock.ip(),
                                    audio_rtcp_recv_sock.port()
                                );
                            }
                            _ => {}
                        }
                    }
                    message => {
                        panic!("{:?}", message);
                    }
                }
            }
        }
    };

    tokio::join!(user_input_fut, event_fut);
}
