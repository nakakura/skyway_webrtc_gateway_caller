use async_trait::async_trait;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

#[cfg(test)]
use mockall::automock;

use crate::domain::peer::value_object::{CreatePeerParams, PeerEventEnum, PeerInfo};

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait PeerRepository: Interface {
    async fn register(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error>;
    async fn erase(&self, peer_info: &PeerInfo) -> Result<(), error::Error>;
}

// WebRTC GatewayのAPIをCallするObjectのInterface
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait PeerRepositoryApi: Interface {
    async fn create(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error>;
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
    async fn close(&self, peer_info: &PeerInfo) -> Result<(), error::Error>;
}
