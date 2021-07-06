use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::media::value_object::{MediaConnectionIdWrapper, MediaId, RtcpId};
use crate::error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait MediaApi: Interface {
    async fn create_media(&self, is_video: Value) -> Result<SocketInfo<MediaId>, error::Error>;
    async fn delete_media(&self, media_id: Value) -> Result<MediaId, error::Error>;
    async fn create_rtcp(&self) -> Result<SocketInfo<RtcpId>, error::Error>;
    async fn delete_rtcp(&self, rtcp_id: Value) -> Result<RtcpId, error::Error>;
    async fn call(&self, call_query: Value) -> Result<MediaConnectionIdWrapper, error::Error>;
}
