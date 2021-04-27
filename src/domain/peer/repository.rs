use async_trait::async_trait;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

#[cfg(test)]
use mockall::automock;

use crate::domain::peer::value_object::{CreatePeerParams, PeerInfo};

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait PeerRepository: Interface {
    async fn register(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error>;
    async fn erase(&self, peer_info: &PeerInfo) -> Result<(), error::Error>;
}
