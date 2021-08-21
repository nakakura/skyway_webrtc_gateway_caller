use async_trait::async_trait;
use shaku::Component;
use skyway_webrtc_gateway_api::peer;

use crate::domain::webrtc::peer::repository::ReerRepositoryApi;
use crate::domain::webrtc::peer::value_object::{
    CreatePeerParams, PeerEventEnum, PeerInfo, PeerStatusMessage,
};
use crate::error;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = ReerRepositoryApi)]
pub(crate) struct PeerRepositoryApiImpl;

impl Default for PeerRepositoryApiImpl {
    fn default() -> Self {
        PeerRepositoryApiImpl {}
    }
}

//FIXME: シンプルなのでUnitテストはしていない
#[async_trait]
impl ReerRepositoryApi for PeerRepositoryApiImpl {
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
