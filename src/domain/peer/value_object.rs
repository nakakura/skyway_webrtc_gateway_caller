use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::domain::peer::repository::PeerRepository;

#[cfg(test)]
use mockall::automock;

pub use skyway_webrtc_gateway_api::peer::PeerEventEnum;
pub use skyway_webrtc_gateway_api::prelude::PeerId;
pub use skyway_webrtc_gateway_api::prelude::PeerInfo;
pub use skyway_webrtc_gateway_api::prelude::Token;

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub base_url: String,
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PeerControlApi: Interface {
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait PeerApi: Interface {
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
}

#[derive(Component)]
#[shaku(interface = PeerApi)]
pub(crate) struct PeerEvent {
    #[shaku(inject)]
    api: Arc<dyn PeerControlApi>,
}

#[async_trait]
impl PeerApi for PeerEvent {
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error> {
        self.api.event(peer_info).await
    }
}

#[cfg(test)]
mod test_peer_event {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::peer::PeerCloseEvent;

    use super::*;
    use crate::di::PeerControlApiContainer;

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
        let mut mock = MockPeerControlApi::default();
        let ret_peer_info = peer_info.clone();
        mock.expect_event().return_once(move |_| {
            Ok(PeerEventEnum::CLOSE(PeerCloseEvent {
                params: ret_peer_info,
            }))
        });

        // object生成の際にmockを埋め込む
        let module = PeerControlApiContainer::builder()
            .with_component_override::<dyn PeerControlApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerApi = module.resolve_ref();

        // execute
        let event = repository.event(peer_info).await;

        // evaluate
        assert_eq!(event.unwrap(), expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // Errorを返すmockを作成
        let mut mock = MockPeerControlApi::default();
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // object生成の際にmockを埋め込む
        let module = PeerControlApiContainer::builder()
            .with_component_override::<dyn PeerControlApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerApi = module.resolve_ref();

        // execute
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let event = repository.event(peer_info).await;

        // evaluate
        if let Err(error::Error::LocalError(message)) = event {
            assert_eq!(message.as_str(), "error");
        } else {
            unreachable!();
        }
    }
}
