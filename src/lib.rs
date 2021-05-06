use futures::stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::ros_service::launch_service;

pub(crate) mod application;
pub(crate) mod di;
pub(crate) mod domain;
pub(crate) mod infra;

pub use application::usecase::peer::create::{CreatePeerSuccessMessage, ErrorMessage};
pub use application::usecase::peer::delete::DeletePeerSuccessMessage;
pub use application::usecase::peer::event::PeerEventMessage;
pub use application::usecase::service::ReturnMessage;
pub use domain::peer::value_object::{PeerEventEnum, PeerId, PeerInfo, Token};
pub use skyway_webrtc_gateway_api::data::{DataConnectionId, DataConnectionIdWrapper};

// FIXME: 未テスト
pub async fn run() -> (
    mpsc::Sender<(oneshot::Sender<String>, String)>,
    mpsc::Receiver<String>,
) {
    // message_tx, message_rx(string, once::channel)生成
    let (message_tx, message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);

    // event_tx, event_rx生成
    let (event_tx, event_rx) = mpsc::channel::<String>(10);

    // message_rxをこの関数で監視
    tokio::spawn(ros_service_observe(message_rx, event_tx));

    (message_tx, event_rx)
}

// FIXME: 未実装
async fn ros_service_observe(
    receiver: mpsc::Receiver<(oneshot::Sender<String>, String)>,
    event_tx: mpsc::Sender<String>,
) {
    // ROS Clientからのメッセージを待機
    // Oneshot Senderで返す

    // ユーザには、eventはevent関数を見に行ってもらう
    // その他の操作はmessage_txにsendしてもらう。処理結果はonce::channelで返す

    // 時々eventの方に返さなければならないものはevent_txに送る
    // 全体通して副作用はここだけ
    // peer, media, dataのステータスを保管

    let receiver = ReceiverStream::new(receiver);
    receiver
        .fold(
            event_tx,
            |event_tx, (message_response_tx, message)| async move {
                let result = application::service_creator::create(message).await;
                message_response_tx.send(serde_json::to_string(&result).unwrap());

                if let ReturnMessage::PEER_CREATE(params) = result {
                    tokio::spawn(event_observe(params.params, event_tx.clone()));
                }

                event_tx
            },
        )
        .await;
}

async fn event_observe(peer_info: PeerInfo, event_tx: mpsc::Sender<String>) {
    use shaku::HasComponent;

    use crate::application::usecase::service::EventListener;
    use crate::di::PeerEventServiceContainer;

    // Mockを埋め込んだEventServiceを生成
    let module = PeerEventServiceContainer::builder().build();
    let event_service: &dyn EventListener = module.resolve_ref();
    let value = serde_json::to_value(&peer_info).unwrap();
    event_service.execute(event_tx, value).await;
}

mod ros_service {
    use super::*;

    pub fn launch_service(message_tx: mpsc::Sender<(oneshot::Sender<String>, String)>) {}
}
