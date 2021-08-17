// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;

use crate::error;

#[cfg(test)]
use mockall::automock;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
pub use skyway_webrtc_gateway_api::peer::{
    PeerCallEvent, PeerCloseEvent, PeerConnectionEvent, PeerErrorEvent, PeerEventEnum, PeerId,
    PeerInfo, PeerOpenEvent, Token,
};

/// POST /peerで必要なパラメータ類
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}

// FIXME: Value Objectではない
#[cfg_attr(test, automock)]
#[async_trait]
pub trait PeerApi: Interface {
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
}

// FIXME: Value Objectではない
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Peer: Interface {
    async fn event(&self, message: Value) -> Result<PeerEventEnum, error::Error>;
}

#[derive(Component)]
#[shaku(interface = Peer)]
pub(crate) struct PeerImpl {
    #[shaku(inject)]
    api: Arc<dyn PeerApi>,
}

#[async_trait]
impl Peer for PeerImpl {
    async fn event(&self, message: Value) -> Result<PeerEventEnum, error::Error> {
        // ドメイン層の知識として、JSONメッセージのParseを行う
        let peer_info = serde_json::from_value::<PeerInfo>(message)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        self.api.event(peer_info).await
    }
}

#[cfg(test)]
mod test_peer_event {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::peer::PeerCloseEvent;

    use super::*;
    use crate::di::PeerEventServiceContainer;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let expected = PeerEventEnum::CLOSE(PeerCloseEvent {
            params: peer_info.clone(),
        });

        // CLOSEイベントを返すmockを作成
        let mut mock = MockPeerApi::default();
        let ret_peer_info = peer_info.clone();
        mock.expect_event().return_once(move |_| {
            Ok(PeerEventEnum::CLOSE(PeerCloseEvent {
                params: ret_peer_info,
            }))
        });

        // object生成の際にmockを埋め込む
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerApi>(Box::new(mock))
            .build();
        let repository: &dyn Peer = module.resolve_ref();

        // execute
        let event = repository
            .event(serde_json::to_value(&peer_info).unwrap())
            .await;

        // evaluate
        assert_eq!(event.unwrap(), expected);
    }

    #[tokio::test]
    async fn fail_api() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // Errorを返すmockを作成
        let mut mock = MockPeerApi::default();
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // object生成の際にmockを埋め込む
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerApi>(Box::new(mock))
            .build();
        let repository: &dyn Peer = module.resolve_ref();

        // execute
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let event = repository
            .event(serde_json::to_value(&peer_info).unwrap())
            .await;

        // evaluate
        if let Err(error::Error::LocalError(message)) = event {
            assert_eq!(message.as_str(), "error");
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn fail_invalid_json() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // その手前のJSON Parseでエラーになるため、このmockは呼ばれない
        let mut mock = MockPeerApi::default();
        mock.expect_event().return_once(move |_| unreachable!());

        // object生成の際にmockを埋め込む
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerApi>(Box::new(mock))
            .build();
        let repository: &dyn Peer = module.resolve_ref();

        // execute
        let event = repository.event(serde_json::Value::Bool(true)).await;

        // evaluate
        if let Err(error::Error::SerdeError { error: _ }) = event {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
