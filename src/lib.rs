use futures::stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

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

// C++側から呼び出される、Rust側のmain関数として動作
// Integration Testでのみテストを行う
pub async fn run() -> (
    mpsc::Sender<(oneshot::Sender<String>, String)>,
    mpsc::Receiver<String>,
) {
    // ROS Serviceは2つ準備する
    // 操作を行うためのskyway_control serviceと、イベントを受信するためのskyway_event serviceである
    // この関数でそれらのサービスとデータをやりとりするためのチャンネルを生成する
    // サービスではJSONメッセージで外部とやり取りするので、String型のチャンネルを生成する

    // skyway_control serviceとやりとりするためのチャンネル
    // 結果の返却のためのoneshot::Sender<String>と、操作メッセージのJSONを受け取るためのmpsc::channelを生成
    // SenderをROS側に渡し、Receiverを監視する
    // ROS終了時にSenderをdropすることでReceiverの監視を停止する
    let (message_tx, message_rx) = mpsc::channel::<(oneshot::Sender<String>, String)>(10);

    // skyway_event serviceとやりとりするためのチャンネル
    // イベント自体は自動的に監視しておき、発生時にevent_txにsendしておく。
    // skyway_event serviceにアクセスがあった場合、event_rxからイベントをpopして返す
    // イベントがない場合は来るまで待機して返す
    // TODO: タイムアウトの仕様を検討する
    let (event_tx, event_rx) = mpsc::channel::<String>(10);

    // message_txの監視を開始
    // messege_txの指示次第でeventを監視する必要が生じるので、event_txも渡す
    // (例: peer objectを生成したらpeer eventの監視を合わせて開始する)
    tokio::spawn(skyway_control_service_observe(message_rx, event_tx));

    // message_tx, event_rxをROS側に返す
    // message_txをskyway_control serviceに、event_rxをskyway_event serviceに結びつける部分はC++側で実装する
    (message_tx, event_rx)
}

// skyway_control serviceからのJSONメッセージを監視し続ける
// これは保持するReceiverが結びついているSenderが破棄されるまで続ける
// Integration Testでのみテスト
async fn skyway_control_service_observe(
    receiver: mpsc::Receiver<(oneshot::Sender<String>, String)>,
    event_tx: mpsc::Sender<String>,
) {
    // 時々eventの方に返さなければならないものはevent_txに送る
    // 全体通して副作用はここだけ
    // peer, media, dataのステータスを保管

    let receiver = ReceiverStream::new(receiver);
    // fold内部で状態を持つ。Rust側のコードで状態を持つのはこのfoldのみに留める
    receiver
        .fold(
            // TODO: ここで必要な状態を保持するHashMapも保持する
            event_tx,
            |event_tx, (message_response_tx, message)| async move {
                // application層のメソッドにメッセージを渡す
                // 内部で適切に各Serviceに振り分けて結果のみ返してもらう
                // エラーが生じた場合も、エラーを示すJSONメッセージが返される(ReturnMessage::ERROR)ので、ROS側に送る
                let result = application::service_creator::create(message).await;
                message_response_tx.send(serde_json::to_string(&result).unwrap());

                // イベントを監視する必要が生じた場合は、イベントの監視を開始する
                // イベントはオブジェクトのCLOSE, ERRORと、ROS側の終了が検知されるまでは監視し続け、
                // 適宜event_txへsendされる
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

    // Event監視のためのServiceを生成
    let module = PeerEventServiceContainer::builder().build();
    let event_service: &dyn EventListener = module.resolve_ref();

    // peer_infoは必ずto_valueに成功するのでunwrapでよい
    let value = serde_json::to_value(&peer_info).unwrap();
    event_service.execute(event_tx, value).await;
}
