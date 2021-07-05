use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::value_object::{DataConnectionEventEnum, DataConnectionIdWrapper, DataId};

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait DataApi: Interface {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error>;
    async fn delete(&self, data_id: Value) -> Result<DataId, error::Error>;
    async fn connect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error>;
    async fn disconnect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error>;
    async fn redirect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error>;
    async fn event(&self, params: Value) -> Result<DataConnectionEventEnum, error::Error>;
}
