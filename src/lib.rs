use futures::stream::StreamExt;
use shaku::HasComponent;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::application::usecase::service::EventListener;
use crate::prelude::ResponseMessageBodyEnum;
pub use application::usecase::value_object::{ResponseMessage, ServiceParams};
pub use domain::peer::value_object::{PeerEventEnum, PeerId, PeerInfo, Token};

pub(crate) mod application;
pub(crate) mod di;
pub(crate) mod domain;
pub mod error;
pub(crate) mod infra;
pub mod prelude;

// ROS Serviceは2つ準備する
// 操作を行うためのskyway_control serviceと、イベントを受信するためのskyway_event serviceである
// この関数でそれらのサービスとデータをやりとりするためのチャンネルを生成する
//
// End-User Application <--> Presentation層間
// End-User ApplicationはJSONもしくはService Paramsで操作指示を行い、
// 結果としてResponseMessageを受け取る
// エンドユーザから受け取ったJSONのパースはPresentation層の責務として処理する
//
// Presentation層 <--> Application層間
// Service Paramsで操作指示を行い、ResponseMessageを受け取る
//
// Application層 <--> Domain層間
// Service Paramsで操作指示を行い、ResponseMessageを受け取る
//
// Domain層, Infra層
// Service Paramsとして与えられた値がDomain知識に適合するかどうかはDomain Objectの責務として判断する
// Domain層 <--> Infra層間のやり取りはDomain Objectを利用する

// Application層以降のパイプラインを組み上げる関数
// Presentation層に渡すチャンネルを返す
//
// なお、Unit Testは行わずIntegration Testでのみテストを行う
pub async fn run(
    base_url: &str,
) -> (
    mpsc::Sender<(oneshot::Sender<ResponseMessage>, ServiceParams)>,
    mpsc::Receiver<ResponseMessage>,
) {
    // skyway webrtc gateway のbase_urlを設定する
    skyway_webrtc_gateway_api::initialize(base_url);

    // skyway_control serviceのデータのやりとり
    // ServiceParamsで与えられた支持に対し、oneshotチャンネルへResponseMessageを返す
    // 副作用としてイベントが発生した場合はskyway_event serviceへ転送し、ここでは返さない
    // ROS終了時にSenderをdropすることでReceiverの監視を停止する
    let (message_tx, message_rx) =
        mpsc::channel::<(oneshot::Sender<ResponseMessage>, ServiceParams)>(10);
    // skyway_event serviceとやりとりするためのチャンネルを生成する
    // イベント監視の必要性が生じた場合は自動で監視を行い、このチャンネルへsendする
    // skyway_event serviceにアクセスがあった場合、event_rxからイベントをpopして返す
    // イベントがない場合は来るまで待機して返す
    // TODO: タイムアウトの仕様を検討する
    let (event_tx, event_rx) = mpsc::channel::<ResponseMessage>(10);

    // message_txの監視を開始
    // messege_txの指示次第でeventを監視する必要が生じるので、event_txも渡す
    // (例: peer objectを生成したらpeer eventの監視を合わせて開始する)
    tokio::spawn(skyway_control_service_observe(message_rx, event_tx));

    // message_tx, event_rxをROS側に返す
    // message_txをskyway_control serviceに、event_rxをskyway_event serviceに結びつける部分はC++側で実装する
    (message_tx, event_rx)
}

// skyway_control serviceからのメッセージ(ServiceParams)を監視し続ける
// これは保持するReceiverが結びついているSenderが破棄されるまで続ける
// crate全体を通して、Stateはこの関数内のfoldのみが保持する。
// Stateとして、各PeerIdに対応するMediaConnectionId, DataConnectionIDなどを保持する
//
// なお、Unit Testは行わずIntegration Testでのみテストを行う
async fn skyway_control_service_observe(
    receiver: mpsc::Receiver<(oneshot::Sender<ResponseMessage>, ServiceParams)>,
    event_tx: mpsc::Sender<ResponseMessage>,
) {
    let receiver = ReceiverStream::new(receiver);
    receiver
        .fold(
            // TODO: ここで必要な状態を保持するHashMapも保持する。StateはこのHashMapのみに留める
            event_tx,
            |event_tx, (message_response_tx, message)| async move {
                // application層のメソッドにメッセージを渡す
                // 内部で適切に各Serviceに振り分けて結果のみ返してもらう
                // エラーが生じた場合も、エラーを示すJSONメッセージが返される(ResponseMessage::ERROR)のでそのままPresentation層へ渡す
                let result = application::service_creator::create(message).await;
                let _ = message_response_tx.send(result.clone());

                // イベントを監視する必要が生じた場合は、イベントの監視を開始する
                // イベントはオブジェクトのCLOSE, ERRORと、ROS側の終了が検知されるまでは監視し続け、
                // 適宜event_txへsendされる
                // FIXME: too long
                match result {
                    ResponseMessage::Success(ResponseMessageBodyEnum::PeerCreate(params)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            use crate::di::PeerEventServiceContainer;
                            let module = PeerEventServiceContainer::builder().build();
                            let event_service: &dyn EventListener = module.resolve_ref();
                            let value = serde_json::to_value(&params).unwrap();
                            event_service.execute(tx, value).await;
                        });
                    }
                    ResponseMessage::Success(ResponseMessageBodyEnum::DataConnect(params)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            use crate::di::DataEventServiceContainer;
                            let module = DataEventServiceContainer::builder().build();
                            let event_service: &dyn EventListener = module.resolve_ref();
                            let value = serde_json::to_value(&params).unwrap();
                            event_service.execute(tx, value).await;
                        });
                    }
                    ResponseMessage::Success(ResponseMessageBodyEnum::DataRedirect(params)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            use crate::di::DataEventServiceContainer;
                            let module = DataEventServiceContainer::builder().build();
                            let event_service: &dyn EventListener = module.resolve_ref();
                            let value = serde_json::to_value(&params).unwrap();
                            event_service.execute(tx, value).await;
                        });
                    }
                    ResponseMessage::Success(ResponseMessageBodyEnum::MediaCall(params)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            use crate::di::MediaEventServiceContainer;
                            let module = MediaEventServiceContainer::builder().build();
                            let event_service: &dyn EventListener = module.resolve_ref();
                            let value = serde_json::to_value(&params.media_connection_id).unwrap();
                            event_service.execute(tx, value).await;
                        });
                    }
                    ResponseMessage::Success(ResponseMessageBodyEnum::MediaAnswer(params)) => {
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            use crate::di::MediaEventServiceContainer;
                            let module = MediaEventServiceContainer::builder().build();
                            let event_service: &dyn EventListener = module.resolve_ref();
                            let value = serde_json::to_value(&params.media_connection_id).unwrap();
                            event_service.execute(tx, value).await;
                        });
                    }
                    _ => {}
                }

                event_tx
            },
        )
        .await;
}
