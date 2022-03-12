use tokio::sync::mpsc;

use common::data;
use common::peer;
use common::terminal;
use module::prelude::*;
use module::run;
use response_message::*;

mod common;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();

    // gatewayを操作するためのメッセージをやり取りするチャンネル
    let (message_tx, mut event_rx) = run("http://localhost:8000").await;

    // peer objectを作成
    let peer_info = peer::create_peer(&message_tx, api_key, "data_caller").await;

    // data socketの開放
    // エンドユーザプログラムはこのポートにデータを流し込む
    let data_socket = data::create_data(&message_tx).await;
    // End User Programでデータを受信するポートを指定
    let recv_socket = SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 9000).unwrap();

    // peerに対してDataConnectionを確立開始する
    let query = ConnectQuery {
        peer_id: peer_info.peer_id(),
        token: peer_info.token(),
        options: None,
        target_id: PeerId::new("data_callee"),
        params: Some(DataIdWrapper {
            data_id: data_socket.get_id().unwrap().clone(),
        }),
        redirect_params: Some(recv_socket.clone()),
    };
    let _data_connection_id = data::connect(&message_tx, query).await;

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
        while let Some(message) = event_rx.recv().await {
            if let ResponseResult::Success(event) = ResponseResult::from_str(&message).unwrap() {
                match event {
                    ResponseMessage::Peer(PeerResponse::Event(PeerEventEnum::ERROR(
                        error_event,
                    ))) => {
                        eprintln!("error recv: {:?}", error_event);
                    }
                    ResponseMessage::Peer(PeerResponse::Event(PeerEventEnum::CLOSE(
                        close_event,
                    ))) => {
                        println!("{:?} has been deleted. \nExiting Program", close_event);
                        break;
                    }
                    ResponseMessage::Data(DataResponse::Event(DataConnectionEventEnum::OPEN(
                        data_connection_id_wrapper,
                    ))) => {
                        // DataConnectionの確立に成功
                        println!(
                            "data connection has been opened: {}",
                            data_connection_id_wrapper.data_connection_id.as_str()
                        );
                        println!(
                            "you can send data to: {}:{}",
                            data_socket.ip(),
                            data_socket.port()
                        );
                        println!(
                            "you can receive data at: {}:{}",
                            recv_socket.ip(),
                            recv_socket.port()
                        );
                    }
                    ResponseMessage::Data(DataResponse::Event(event)) => {
                        println!("data event: {:?}", event);
                    }
                    event => {
                        println!("recv event: {:?}", event);
                    }
                }
            }
        }
    };

    tokio::join!(user_input_fut, event_fut);
}
