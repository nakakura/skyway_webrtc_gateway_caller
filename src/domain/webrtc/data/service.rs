use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::data::value_object::{
    DataConnectionEventEnum, DataConnectionIdWrapper, DataId,
};
use crate::error;

#[cfg(test)]
use mockall::automock;

/// /data APIに対応する機能を定義する
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
