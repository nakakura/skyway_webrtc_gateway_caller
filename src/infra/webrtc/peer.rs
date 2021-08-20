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
