use async_trait::async_trait;
use shaku::Interface;

use crate::domain::webrtc::peer::value_object::{
    CreatePeerParams, PeerEventEnum, PeerInfo, PeerStatusMessage,
};
use crate::error;

#[cfg(test)]
use mockall::automock;

/// /data APIに対応する機能を定義する
#[cfg_attr(test, automock)]
#[async_trait]
pub trait PeerRepository: Interface {
    async fn create(&self, params: CreatePeerParams) -> Result<PeerInfo, error::Error>;
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
    async fn status(&self, peer_info: &PeerInfo) -> Result<PeerStatusMessage, error::Error>;
    async fn delete(&self, peer_info: &PeerInfo) -> Result<(), error::Error>;
}
