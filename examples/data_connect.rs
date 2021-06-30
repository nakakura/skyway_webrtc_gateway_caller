mod common;

use rust_module::prelude::*;
use rust_module::run;

use common::data;
use common::peer;

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

    let data_connection_id =
        data::connect(&message_tx, &peer_info, data_socket.get_id().unwrap()).await;

    /*
    // 終了処理
    // socketの開放
    // PeerObjectで利用した場合は、PeerObjectの削除時に開放されるので、特に必要ない
    let _closed_data_id = data::delete_data(&message_tx, data_socket.get_id().unwrap()).await;

    peer::delete_peer(&message_tx, &peer_info).await;
    let event = event_rx.recv().await;
    println!("event {:?}", event);
     */
}
