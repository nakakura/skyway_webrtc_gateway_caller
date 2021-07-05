use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::media::value_object::MediaId;
use crate::error;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait MediaApi: Interface {
    async fn create_media(&self, is_video: Value) -> Result<SocketInfo<MediaId>, error::Error>;
    async fn delete_media(&self, media_id: Value) -> Result<MediaId, error::Error>;
}
