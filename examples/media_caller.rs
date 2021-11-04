use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use common::media;
use common::peer;
use common::terminal;
use response_message::*;
use module::prelude::*;
use module::run;

mod common;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();

    // gatewayを操作するためのメッセージをやり取りするチャンネル
    let (message_tx, mut event_rx) = run("http://localhost:8000").await;

    // peer objectを作成
    let peer_info = peer::create_peer(&message_tx, api_key, "media_caller").await;

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
                "call" => {}
                _ => {}
            }
        }
    };

    // media socketの開放
    // video送信用ポート
    let media_socket_video = media::create_media(&message_tx, true).await;
    // audio送信用ポート
    let media_socket_audio = media::create_media(&message_tx, false).await;
    // rtcp送信用ポート
    let rtcp_socket = media::create_rtcp(&message_tx, true).await;

    // 受信用ポート
    let video_recv_sock = SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 13000).unwrap();
    let video_rtcp_recv_sock =
        SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 13001).unwrap();
    let audio_recv_sock = SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 13002).unwrap();
    let audio_rtcp_recv_sock =
        SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 13003).unwrap();

    // eventを出力する
    let event_fut = async {
        while let Some(message) = event_rx.next().await {
            if let ResponseMessage::Success(event) = ResponseMessage::from_str(&message).unwrap() {
                match event {
                    ResponseMessageBodyEnum::Peer(PeerResponseMessageBodyEnum::Event(
                        PeerEventEnum::ERROR(error_event),
                    )) => {
                        eprintln!("error recv: {:?}", error_event);
                    }
                    ResponseMessageBodyEnum::Peer(PeerResponseMessageBodyEnum::Event(
                        PeerEventEnum::CLOSE(close_event),
                    )) => {
                        println!("{:?} has been deleted. \nExiting Program", close_event);
                        break;
                    }
                    ResponseMessageBodyEnum::Media(MediaResponseMessageBodyEnum::Event(event)) => {
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

    let call_query = CallQuery {
        peer_id: peer_info.peer_id(),
        token: peer_info.token(),
        target_id: PeerId("media_callee".into()),
        constraints: Some(Constraints {
            video: true,
            videoReceiveEnabled: None,
            audio: true,
            audioReceiveEnabled: None,
            video_params: Some(MediaParams {
                band_width: 1500,
                codec: "H264".to_string(),
                media_id: media_socket_video.get_id().unwrap(),
                rtcp_id: Some(rtcp_socket.get_id().unwrap()),
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
        }),
        redirect_params: Some(RedirectParameters {
            video: Some(video_recv_sock.clone()),
            video_rtcp: Some(video_rtcp_recv_sock.clone()),
            audio: Some(audio_recv_sock.clone()),
            audio_rtcp: Some(audio_rtcp_recv_sock.clone()),
        }),
    };

    let _media_connection_id = media::call(&message_tx, call_query).await;

    tokio::join!(user_input_fut, event_fut);
}
