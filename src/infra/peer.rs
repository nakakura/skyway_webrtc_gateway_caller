use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;
use skyway_webrtc_gateway_api::error;
use skyway_webrtc_gateway_api::peer;

use crate::domain::peer::repository::PeerRepository;
use crate::domain::peer::value_object::{CreatePeerParams, PeerEventEnum, PeerInfo};

#[cfg(test)]
use mockall::*;

// WebRTC GatewayのAPIをCallするObjectのInterface
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait PeerRepositoryApi: Interface {
    async fn create(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error>;
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
    async fn close(&self, peer_info: &PeerInfo) -> Result<(), error::Error>;
}

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = PeerRepositoryApi)]
pub(crate) struct PeerRepositoryApiImpl;

impl Default for PeerRepositoryApiImpl {
    fn default() -> Self {
        PeerRepositoryApiImpl {}
    }
}

// シンプルなのでテストはしていない
#[async_trait]
impl PeerRepositoryApi for PeerRepositoryApiImpl {
    async fn create(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error> {
        peer::create(&params.key, &params.domain, params.peer_id, params.turn).await
    }

    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error> {
        peer::event(peer_info.clone()).await
    }

    async fn close(&self, peer_info: &PeerInfo) -> Result<(), error::Error> {
        peer::delete(peer_info).await
    }
}

// PeerRepositoryの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = PeerRepository)]
pub(crate) struct PeerRepositoryImpl {
    #[shaku(inject)]
    api: Arc<dyn PeerRepositoryApi>,
}

#[async_trait]
impl PeerRepository for PeerRepositoryImpl {
    async fn register(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error> {
        // skyway_webrtc_gateway_api crate must be initialized by giving the base_url only once.
        // It doesn't matter how many times you give it, it won't make an error.
        // And since SkyWay can't do anything without creating a peer object,
        // I decided to give it the value here.
        skyway_webrtc_gateway_api::initialize(&params.base_url);
        // The information is returned for accessing the Peer Object on the SkyWay WebRTC Gateway.
        // At this point, the Peer Object is not yet registered with the SkyWay server.
        let peer_info = self.api.create(params).await?;
        // The PeerInfo is returned for using PeerObjects registered with the SkyWay server.
        // SkyWay functions can be used now.
        // The parameters of PeerInfo itself, such as PeerId and Token,
        // are basically the same as those obtained by create method.
        loop {
            let event = self.api.event(peer_info.clone()).await?;
            match event {
                PeerEventEnum::TIMEOUT => continue,
                PeerEventEnum::OPEN(o) => return Ok(o.params),
                _ => {
                    return Err(error::Error::LocalError(
                        "Receive an event other than open".into(),
                    ))
                }
            }
        }
    }

    async fn erase(&self, peer_info: &PeerInfo) -> Result<(), error::Error> {
        // WebRTC Gatewayに削除指示を出す
        let _ = self.api.close(peer_info).await?;
        // 削除完了を確認
        loop {
            let event = self.api.event(peer_info.clone()).await?;
            match event {
                PeerEventEnum::TIMEOUT => continue,
                PeerEventEnum::CLOSE(_) => return Ok(()),
                _ => {
                    return Err(error::Error::LocalError(
                        "Receive an event other than close".into(),
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod test_create {
    use skyway_webrtc_gateway_api::peer::{PeerCloseEvent, PeerOpenEvent};

    use super::*;
    use crate::di::PeerRepositoryContainer;
    use crate::domain::peer::value_object::PeerId;

    fn create_peer_info() -> PeerInfo {
        PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap()
    }

    fn create_params() -> CreatePeerParams {
        CreatePeerParams {
            base_url: "http://localhost:8000".into(),
            key: "api_key".into(),
            domain: "localhost".into(),
            peer_id: PeerId::new("peer_id"),
            turn: true,
        }
    }

    #[tokio::test]
    async fn register_success() {
        // create params
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();

        // set up create method
        // WebRTC GatewayにPeer Objectの生成指示を行うことに成功したケース
        let ret_peer_info = peer_info.clone();
        mock.expect_create().return_once(move |_| Ok(ret_peer_info));

        // set up event method
        // すぐOPENイベントを受信したケース
        let ret_peer_info = peer_info.clone();
        mock.expect_event().return_once(move |_| {
            Ok(PeerEventEnum::OPEN(PeerOpenEvent {
                params: ret_peer_info,
            }))
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.register(create_params()).await;

        // evaluate
        assert_eq!(result.unwrap(), peer_info);
    }

    #[tokio::test]
    async fn register_success_after_timeout() {
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();

        // set up create method
        // WebRTC GatewayにPeer Objectの生成指示を行うことに成功したケース
        let ret_peer_info = peer_info.clone();
        mock.expect_create().return_once(move |_| Ok(ret_peer_info));

        // set up event method
        // 一回TimeoutしたあとにOPENイベントが来たケース
        let mut counter = 0u8;
        mock.expect_event().returning(move |_| {
            if counter == 0 {
                counter += 1;
                Ok(PeerEventEnum::TIMEOUT)
            } else {
                let peer_info = create_peer_info();
                Ok(PeerEventEnum::OPEN(PeerOpenEvent { params: peer_info }))
            }
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.register(create_params()).await;

        // evaluate
        assert_eq!(result.unwrap(), peer_info);
    }

    #[tokio::test]
    async fn register_fail_wrong_event() {
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();
        // set up create method
        // WebRTC GatewayにPeer Objectの生成指示を行うことに成功したケース
        let ret_peer_info = peer_info.clone();
        mock.expect_create().return_once(move |_| Ok(ret_peer_info));

        // set up event method
        // OPEN Eventが来るべきところ、CLOSE Eventが来るケース
        let ret_peer_info = peer_info.clone();
        mock.expect_event().return_once(move |_| {
            Ok(PeerEventEnum::CLOSE(PeerCloseEvent {
                params: ret_peer_info,
            }))
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.register(create_params()).await;

        // evaluate
        if let Err(error::Error::LocalError(err)) = result {
            assert_eq!(err.as_str(), "Receive an event other than open");
        } else {
            unreachable!();
        }
    }
    #[tokio::test]
    async fn register_err_create_api_failed() {
        // create mock
        let mut mock = MockPeerRepositoryApi::default();

        // set up create method
        // WebRTC Gatewayのcreate APIのコールに失敗
        mock.expect_create()
            .return_once(move |_| Err(error::Error::LocalError("error".into())));

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.register(create_params()).await;

        // evaluate
        if let Err(error::Error::LocalError(err)) = result {
            assert_eq!(err.as_str(), "error");
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn register_failed_event_api_error() {
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();

        // set up create method
        // WebRTC GatewayにPeer Objectの生成指示を行うことに成功したケース
        let ret_peer_info = peer_info.clone();
        mock.expect_create().return_once(move |_| Ok(ret_peer_info));

        // set up event method
        // WebRTC Gatewayのevent APIのコールに失敗
        mock.expect_event()
            .return_once(move |_| Err(error::Error::LocalError("error".into())));

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.register(create_params()).await;

        // evaluate
        if let Err(error::Error::LocalError(err)) = result {
            assert_eq!(err.as_str(), "error");
        } else {
            unreachable!();
        }
    }
}

#[cfg(test)]
mod test_close {
    use skyway_webrtc_gateway_api::peer::{PeerCloseEvent, PeerOpenEvent};

    use super::*;
    use crate::di::PeerRepositoryContainer;

    fn create_peer_info() -> PeerInfo {
        PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap()
    }

    #[tokio::test]
    async fn success() {
        // create params
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();
        // set up close method
        mock.expect_close().return_once(move |_| Ok(()));
        // set up event method
        let ret_peer_info = peer_info.clone();
        mock.expect_event().return_once(move |_| {
            Ok(PeerEventEnum::CLOSE(PeerCloseEvent {
                params: ret_peer_info,
            }))
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.erase(&peer_info).await;

        // evaluate
        assert_eq!(result.unwrap(), ());
    }

    #[tokio::test]
    async fn success_after_timeout() {
        // create params
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();
        // set up close method
        mock.expect_close().return_once(move |_| Ok(()));
        // set up event method
        let mut counter = 0u8;
        mock.expect_event().returning(move |_| {
            if counter == 0 {
                counter += 1;
                Ok(PeerEventEnum::TIMEOUT)
            } else {
                let peer_info = create_peer_info();
                Ok(PeerEventEnum::CLOSE(PeerCloseEvent { params: peer_info }))
            }
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.erase(&peer_info).await;

        // evaluate
        assert_eq!(result.unwrap(), ());
    }

    #[tokio::test]
    async fn close_api_failed() {
        // create params
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();
        // set up close method
        mock.expect_close()
            .return_once(move |_| Err(error::Error::create_local_error("error")));
        // set up event method
        mock.expect_event().return_once(move |_| {
            unreachable!();
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.erase(&peer_info).await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "error");
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn event_api_failed() {
        // create params
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();
        // set up close method
        mock.expect_close().return_once(move |_| Ok(()));
        // set up event method
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.erase(&peer_info).await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "error");
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn recv_wrong_event() {
        // create params
        let peer_info = create_peer_info();

        // create mock
        let mut mock = MockPeerRepositoryApi::default();
        // set up close method
        mock.expect_close().return_once(move |_| Ok(()));
        // set up event method
        let ret_peer_info = peer_info.clone();
        mock.expect_event().return_once(move |_| {
            Ok(PeerEventEnum::OPEN(PeerOpenEvent {
                params: ret_peer_info,
            }))
        });

        // create target obj
        let module = PeerRepositoryContainer::builder()
            .with_component_override::<dyn PeerRepositoryApi>(Box::new(mock))
            .build();
        let repository: &dyn PeerRepository = module.resolve_ref();

        // execute
        let result = repository.erase(&peer_info).await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "Receive an event other than close");
        } else {
            unreachable!();
        }
    }
}
