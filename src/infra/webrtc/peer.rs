use async_trait::async_trait;
use shaku::Component;
use skyway_webrtc_gateway_api::peer;

use crate::domain::webrtc::peer::entity::{CreatePeerParams, PeerEventEnum, PeerStatusMessage};
use crate::domain::webrtc::peer::repository::PeerRepository;
use crate::domain::webrtc::peer::value_object::PeerInfo;
use crate::error;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = PeerRepository)]
pub(crate) struct PeerRepositoryImpl;

impl Default for PeerRepositoryImpl {
    fn default() -> Self {
        PeerRepositoryImpl {}
    }
}

//FIXME: シンプルなのでUnitテストはしていない
#[async_trait]
impl PeerRepository for PeerRepositoryImpl {
    async fn create(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error> {
        peer::create(&params.key, &params.domain, params.peer_id, params.turn).await
    }

    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error> {
        peer::event(peer_info.clone()).await
    }

    async fn status(&self, peer_info: &PeerInfo) -> Result<PeerStatusMessage, error::Error> {
        peer::status(peer_info).await
    }

    async fn delete(&self, peer_info: &PeerInfo) -> Result<(), error::Error> {
        peer::delete(peer_info).await
    }
}
