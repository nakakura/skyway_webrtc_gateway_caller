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

    // eventを出力する
    let event_fut = async {
        while let Some(ResponseMessage::Success(event)) = event_rx.recv().await {
            match event {
                ResponseMessageBodyEnum::PeerEvent(PeerEventEnum::ERROR(error_event)) => {
                    eprintln!("error recv: {:?}", error_event);
                }
                ResponseMessageBodyEnum::PeerEvent(PeerEventEnum::CLOSE(close_event)) => {
                    println!("{:?} has been deleted. \nExiting Program", close_event);
                    break;
                }
                ResponseMessageBodyEnum::MediaEvent(event) => {
                    println!("media event \n {:?}", event);
                }
                message => {
                    panic!("{:?}", message);
                }
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

    let call_query = CallQuery {
        peer_id: peer_info.peer_id(),
        token: peer_info.token(),
        target_id: PeerId("media_callee".into()),
        constraints: Some(Constraints {
            video: false,
            videoReceiveEnabled: None,
            audio: false,
            audioReceiveEnabled: None,
            video_params: Some(MediaParams {
                band_width: 0,
                codec: "H264".to_string(),
                media_id: media_socket_video.get_id().unwrap(),
                rtcp_id: Some(rtcp_socket.get_id().unwrap()),
                payload_type: None,
                sampling_rate: None,
            }),
            audio_params: None,
            metadata: None,
        }),
        redirect_params: None,
    };

    let _media_connection_id = media::call(&message_tx, call_query).await;

    tokio::join!(user_input_fut, event_fut);
}
