use async_trait::async_trait;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::value_object::DataId;

#[async_trait]
pub(crate) trait DataApi: Interface {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error>;
}
