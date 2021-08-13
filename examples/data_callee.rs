mod common;

use rust_module::prelude::*;
use rust_module::run;
use tokio::sync::mpsc;

use common::data;
use common::peer;
use common::terminal;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("API_KEY").unwrap();

    // gatewayを操作するためのメッセージをやり取りするチャンネル
    let (message_tx, mut event_rx) = run("http://localhost:8000").await;

    // peer objectを作成
    let peer_info = peer::create_peer(&message_tx, api_key, "data_callee").await;

    // data socketの開放
    // データの送信のためのポートの割当
    let data_socket = data::create_data(&message_tx).await;
    // End User Programでデータを受信するポートを指定
    let recv_socket = SocketInfo::<PhantomId>::try_create(None, "127.0.0.1", 9000).unwrap();

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
                    PeerEventEnum::CONNECTION(connect_event),
                )) => {
                    // 相手からDataConnectionの確立が行われた
                    // 確立自体はこの時点で既に完了しているので、データの転送の設定が必要
                    let data_connection_id = connect_event.data_params.data_connection_id;

                    let redirect_params = data::RedirectParams {
                        data_connection_id: data_connection_id,
                        feed_params: Some(DataIdWrapper {
                            data_id: data_socket.get_id().unwrap().clone(),
                        }),
                        redirect_params: Some(recv_socket.clone()),
                    };
                    let _ = data::redirect(&message_tx, redirect_params).await;
                }
                ResponseMessageBodyEnum::Peer(PeerResponseMessageBodyEnum::Event(
                    PeerEventEnum::CLOSE(close_event),
                )) => {
                    println!("{:?} has been deleted. \nExiting Program", close_event);
                    break;
                }
                ResponseMessageBodyEnum::Data(DataResponseMessageBodyEnum::Event(
                    DataConnectionEventEnum::OPEN(data_connection_id),
                )) => {
                    println!(
                        "data connection has been opened: {}",
                        data_connection_id.as_str()
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
                ResponseMessageBodyEnum::Data(DataResponseMessageBodyEnum::Event(event)) => {
                    println!("data event: {:?}", event);
                }
                event => {
                    println!("recv event: {:?}", event);
                }
            }
        }
    };

    tokio::join!(user_input_fut, event_fut);
}
