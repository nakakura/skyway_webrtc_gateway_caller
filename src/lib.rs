// # プログラムの全体構造
// SkyWay WebRTC GatewayをコントロールするためのJSONメッセージのやりとりは、
// skyway-webrtc-gateway-api crateを利用することでRustから実施することができるが、
// イベントのsubscribeなどロジック部分はむき出しであり、SkyWay WebRTC Gatewayの挙動を熟知していないと利用できない。
// このcrateでは、ロジックの隠蔽を行う。
// 操作指示用のSenderを1つ、イベント受信用のReceiverを1つ提供し、これらを通じてメッセージをやり取りするだけで操作できるようにする。
// 内部構造はドメイン駆動の考え方に基づき整理する。
// また、crate全体を通してステートレスな設計にし、将来Stateが必要になった場合もcontrol関数内のfoldのみが保持するよう設計する。

// ## Presentation層
// Presentation層の役割を果たすのは、
// 操作メッセージを与えるためのtokio::sync::mpsc::channel::Senderと
// イベントを受け取るためのtokio::sync::mpsc::channel::Receiverである
// 本crateの要件はこれだけで満たせるので、lib.rs内で生成して返すのみである。
// メッセージを受け取るためにNetworkやROSなどのAPI化を行うためにラッピングする
// crate利用者側の際コードが実質的にPresentation層の役割を果たす
//
// ### End-User Application <--> Presentation層間の通信
// 上記のSender, Receiverでメッセージのやり取りをすることで行う
// メッセージの実体はApplication層で定義されるDTOで、
// SenderにはServiceParams(及び操作の`一次的な結果`を受け取るためのtokio::sync::oneshot)が与えられ、
// ReceiverからはResponseMessageが返される。
// (SkyWay WebRTC Gatewayは、APIに対して処理依頼を行った際に、長時間の処理が必要な場合は値を即時返さない仕様になっている。
// その場合、まずは処理開始できるかどうかだけが返される。上記の`一次的な結果`とはこのメッセージを指す。
// 短時間で処理が完了するAPI callの場合は、`一次的な結果`のみで完結する。)
// これらの与えられた値がJSONのフォーマットを取っていることは、Rustの型システムによって保証されているので、
// Presentation層ではチェックを行わない。

// ## Application層
// src/application以下に配置する
// SkyWay WebRTC Gateawyの各APIに対応するUseCaseと、
// 受け取ったメッセージに対応する適切なUseCase objectを生成するcreatorを実装する。
// 各UseCaseは、application/usecase/service.rsで定義されるService traitに従い実装される。
//
// ### Presentation層 <--> Application層間の通信
// #### 操作指示:
// Presentation層として生成されるSenderからメッセージを受け取る。
// このメッセージはapplication::runに渡され、対応したUseCase objectが生成され、実行される
// End-Userから受け取ったJSONメッセージが各UseCaseに適合したものかどうかはApplication層でチェックし、
// 間違っている場合はErrorメッセージを返す。
// 正しい場合はパラメータを取り出し、Domain層に与える。
// #### イベント:
// イベント監視を開始すべきタイミングをEnd-Userが意識しなくてすむよう、lib.rs内で自動的にイベント監視を開始する。
// イベント監視はevent UseCaseの実行という形で行い、その戻り値はPresentation層のReceiverを通してEnd-Userに返される。

// ## Domain層
// SkyWay WebRTC Gatewayを操作するためのドメイン知識を実装する。
// SkyWay WebRTC GatewayのAPIは大きく/peer, /data, /mediaに分かれているので、
// それぞれのAPIに対応するコードを格納するディレクトリとしてdomain/peer, domain/data, domain/mediaがあり、
// これらの中で共通的に利用されるコードはdomain/commonに格納される。
// ドメイン知識として、SkyWay WebRTC Gatewayの各APIと、それらが利用する値のフォーマットを有する。
// 各APIの機能は、それぞれのディレクトリ内のservice.rs内でtraitとして定義する。
// Application層から与えられたパラメータのチェックは、Domain Objectに与えることでなされる。
//
// このチェックはskyway-webrtc-gateway-api crateで実装されているので、それを内部的に利用する。
// そのためDomain Objectの多くは、skyway-webrtc-gateway-api crate内で定義されている。
// skyway-webrtc-gateway-api crateに対する直接的な依存は、infra層を除けば、
// lib.rs内での初期化と、これらのDomain Objectのみである。
// (domain/*/value_object.rs内のみに留め、pub useする形で自身のobjectとして利用する)
//
// ### Application層 <--> Domain層間の通信
// 各UseCaseが、与えられたJSONメッセージからパラメータを取り出し、対応するDomain層のobjectに与える。

// ## Infra層
// skyway-webrtc-gateway-api crateに依存しており、APIを直接叩く。
//
// ### Domain層 <--> Infra層間の通信
// Domain層はDomain ObjectをInfra層の関数に与え、
// Infra層はskyway-webrtc-gateway-api crateのAPIから返される戻り値をDomain Objectに変換して返す。

use futures::stream::StreamExt;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

pub use application::usecase::value_object::{ResponseMessage, ServiceParams};
pub use domain::webrtc::peer::value_object::{PeerEventEnum, PeerId, PeerInfo, Token};

pub(crate) mod application;
pub(crate) mod di;
pub(crate) mod domain;
pub mod error;
pub(crate) mod infra;
pub mod prelude;

// Presentation層としてchannelを生成し、Application層以降のパイプラインを組み上げる関数。
// 外部から直接的に呼ばれるのはこの関数のみである。
//
// なお、Unit Testは行わずIntegration Testでのみテストを行う
pub async fn run(
    base_url: &str,
) -> (
    mpsc::Sender<(oneshot::Sender<ResponseMessage>, ServiceParams)>,
    mpsc::Receiver<ResponseMessage>,
) {
    // skyway-webrtc-gateway crateにbase_urlを与え、初期化する
    skyway_webrtc_gateway_api::initialize(base_url);

    // End-Userに渡すSenderの生成
    // End-UserはServiceParamsと、oneshotチャネルをこのSenderで与える。
    // 本crateはServiceParamsに対応したUseCaseでの処理を開始し、`一次的な結果`をoneshotチャネルへ返す。
    let (message_tx, message_rx) =
        mpsc::channel::<(oneshot::Sender<ResponseMessage>, ServiceParams)>(10);
    // End-Userに渡すReceiverの生成
    // UseCaseでの処理の結果が`一次的な結果`に留まらず、副作用としてイベント監視の必要性が生じた場合は、
    // このReceiverを介してイベントをEnd-Userに返す。
    // TODO: タイムアウトの仕様を検討する
    let (event_tx, event_rx) = mpsc::channel::<ResponseMessage>(10);

    // Senderの監視を開始する。
    // 副作用としてイベントを返すケースのため、event_txも渡す
    // (例: peer objectを生成したらpeer eventの監視を合わせて開始する)
    tokio::spawn(skyway_control_service_observe(message_rx, event_tx));

    // Presentation層として動作するSender, ReceiverをEnd-Userへ渡す
    (message_tx, event_rx)
}

// End-Userからのメッセージ(ServiceParams)を監視し続ける
// これはEnd-UserがSenderが破棄するまで続ける。
// crate全体を通してステートレスに設計し、将来Stateが必要になった場合もこの関数内のfoldのみに留める
//
// なお、Unit Testは行わずIntegration Testでのみテストを行う
async fn skyway_control_service_observe(
    receiver: mpsc::Receiver<(oneshot::Sender<ResponseMessage>, ServiceParams)>,
    event_tx: mpsc::Sender<ResponseMessage>,
) {
    let receiver = ReceiverStream::new(receiver);
    receiver
        .fold(
            event_tx,
            |event_tx, (message_response_tx, message)| async move {
                // UseCaseを生成し、実行する。
                let result = application::run(message).await;
                // oneshot channelを介して`一次的な結果`を返す。
                // エラーが生じた場合も、エラーを示すJSONメッセージが返される(ResponseMessage::ERROR)のでそのままPresentation層へ渡す
                let _ = message_response_tx.send(result.clone());

                // イベントを監視する必要が生じた場合は、イベントの監視を開始する
                // イベントはオブジェクトのCLOSE, ERRORと、ROS側の終了が検知されるまでは監視し続け、
                // 適宜event_txへsendされる
                if let ResponseMessage::Success(message) = result {
                    if let Some((value, service)) =
                        application::usecase::value_object::event_factory(message)
                    {
                        // event_txをイベント監視スレッドにmoveし、監視を開始する
                        let tx = event_tx.clone();
                        tokio::spawn(async move {
                            service.execute(tx, value).await;
                        });
                    }
                }

                event_tx
            },
        )
        .await;
}
