use async_trait::async_trait;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::value_object::DataId;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait DataApi: Interface {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error>;
}
