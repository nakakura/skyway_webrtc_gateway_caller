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

pub(crate) struct Peer {
    repository: Arc<dyn PeerRepository>,
    peer_info: PeerInfo,
}

#[cfg_attr(test, automock)]
impl Peer {
    pub(crate) fn new(repository: Arc<dyn PeerRepository>, peer_info: PeerInfo) -> Self {
        Self {
            repository,
            peer_info,
        }
    }

    pub(crate) async fn try_create_local(
        repository: Arc<dyn PeerRepository>,
        params: &str,
    ) -> Result<Self, error::Error> {
        // この位置でドメイン層の知識としてJSONの値をチェックする
        let peer_info = serde_json::from_str::<PeerInfo>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?;

        Ok(Self {
            repository,
            peer_info,
        })
    }

    pub(crate) async fn try_create(
        repository: Arc<dyn PeerRepository>,
        params: &str,
    ) -> Result<Self, error::Error> {
        // この位置でドメイン層の知識としてJSONの値をチェックする
        let params = serde_json::from_str::<CreatePeerParams>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?;

        // WebRTC Gateway経由でSkyWayにregistrationを行う
        let peer_info = repository.register(params).await?;

        Ok(Peer::new(repository, peer_info))
    }

    // SkyWay上のPeerObjectの削除に失敗した場合でもLocalのPeerObjectは削除される
    // SkyWay上のPeerObjectが既に存在しない場合は、ローカルのPeerObjectも不要なのでこの挙動で問題ない
    // SkyWayと通信できない場合も、ローカルのPeerObjectは不要なので、この挙動で問題ない
    pub(crate) async fn delete(self) -> Result<PeerInfo, error::Error> {
        let _ = self.repository.erase(&self.peer_info).await?;
        Ok(self.peer_info)
    }

    pub fn peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

#[cfg(test)]
mod test_peer_create {
    use super::*;
    use crate::domain::peer::repository::MockPeerRepository;

    fn create_valid_json_message() -> &'static str {
        r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#
    }

    #[tokio::test]
    async fn success() {
        // create parameter
        let message = create_valid_json_message();

        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_register().returning(|_| {
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308")
        });

        // execute
        let result = Peer::try_create(Arc::new(mock), message).await;

        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // evaluate
        assert_eq!(result.unwrap().peer_info(), &expected);
    }

    #[tokio::test]
    async fn parameter_wrong() {
        // create parameter
        let invalid_message = r#"{
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#;

        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_register().returning(|_| {
            unreachable!();
        });

        // execute
        let result = Peer::try_create(Arc::new(mock), invalid_message).await;

        // evaluate
        if let Err(error::Error::SerdeError { error: e }) = result {
            let message = format!("{:?}", e);
            assert_eq!(
                message.as_str(),
                "Error(\"missing field `base_url`\", line: 6, column: 9)"
            );
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn register_failed() {
        // create parameter
        let message = create_valid_json_message();

        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_register()
            .returning(|_| Err(error::Error::create_local_error("error")));

        // execute
        let result = Peer::try_create(Arc::new(mock), message).await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "error");
        } else {
            unreachable!();
        }
    }
}

#[cfg(test)]
mod test_peer_create_local {
    use super::*;
    use crate::domain::peer::repository::MockPeerRepository;

    #[tokio::test]
    async fn success() {
        // create parameter
        let message = r#"{
            "peer_id": "peer_id",
            "token": "pt-9749250e-d157-4f80-9ee2-359ce8524308"
        }"#;

        // setup mock
        let mock = MockPeerRepository::default();

        // execute
        let result = Peer::try_create_local(Arc::new(mock), message).await;

        // evaluate
        assert_eq!(
            result.unwrap().peer_info(),
            &PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap()
        );
    }
}

#[cfg(test)]
mod test_peer_delete {
    use super::*;
    use crate::domain::peer::repository::MockPeerRepository;

    fn create_peer_info() -> PeerInfo {
        PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap()
    }

    #[tokio::test]
    async fn success() {
        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_erase().returning(|_| Ok(()));

        let peer = Peer::new(Arc::new(mock), create_peer_info());
        // execute
        let result = peer.delete().await;

        let expected = create_peer_info();

        // evaluate
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn fail() {
        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_erase()
            .returning(|_| Err(error::Error::create_local_error("error")));

        let peer = Peer::new(Arc::new(mock), create_peer_info());
        // execute
        let result = peer.delete().await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "error");
        } else {
            unreachable!();
        }
    }
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

    /*
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

     */
}
