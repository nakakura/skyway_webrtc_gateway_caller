use async_trait::async_trait;
use skyway_webrtc_gateway_api::peer;

use crate::domain::webrtc::peer_refactor::repository::PeerRepositoryApi;
use crate::domain::webrtc::peer_refactor::value_object::{
    CreatePeerParams, PeerEventEnum, PeerInfo,
};
use crate::error;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
pub(crate) struct PeerRepositoryApiImpl;

impl Default for PeerRepositoryApiImpl {
    fn default() -> Self {
        PeerRepositoryApiImpl {}
    }
}

//FIXME: シンプルなのでUnitテストはしていない
#[async_trait]
impl PeerRepositoryApi for PeerRepositoryApiImpl {
    async fn create(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error> {
        peer::create(&params.key, &params.domain, params.peer_id, params.turn).await
    }

    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error> {
        peer::event(peer_info.clone()).await
    }

    async fn delete(&self, peer_info: &PeerInfo) -> Result<(), error::Error> {
        peer::delete(peer_info).await
    }
}
