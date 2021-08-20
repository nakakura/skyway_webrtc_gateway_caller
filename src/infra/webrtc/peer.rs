use std::sync::Arc;

use crate::error;
use async_trait::async_trait;
use shaku::*;
use skyway_webrtc_gateway_api::peer;

use crate::domain::webrtc::peer::repository::{PeerRepository, PeerRepositoryApi};
use crate::domain::webrtc::peer::value_object::{
    CreatePeerParams, PeerApi, PeerEventEnum, PeerInfo,
};

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
        Ok(())
    }
}

#[cfg(test)]
mod test_close {
    use skyway_webrtc_gateway_api::peer::PeerCloseEvent;

    use super::*;
    use crate::di::PeerDeleteServiceContainer;
    use crate::domain::webrtc::peer::repository::MockPeerRepositoryApi;

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
        let module = PeerDeleteServiceContainer::builder()
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
        let module = PeerDeleteServiceContainer::builder()
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
        let module = PeerDeleteServiceContainer::builder()
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
}

// PeerApiの具象Struct
#[derive(Component)]
#[shaku(interface = PeerApi)]
pub(crate) struct PeerApiImpl;

impl Default for PeerApiImpl {
    fn default() -> Self {
        PeerApiImpl {}
    }
}

// シンプルなのでテストはしていない
#[async_trait]
impl PeerApi for PeerApiImpl {
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error> {
        peer::event(peer_info).await
    }
}
