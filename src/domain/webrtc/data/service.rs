use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::data::value_object::{
    ConnectQuery, DataConnectionEventEnum, DataConnectionId, DataConnectionIdWrapper,
    DataConnectionStatus, DataId,
};
use crate::error;

#[cfg(test)]
use mockall::automock;

/// /data APIに対応する機能を定義する
#[cfg_attr(test, automock)]
#[async_trait]
pub trait DataApi: Interface {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error>;
    async fn delete(&self, data_id: DataId) -> Result<DataId, error::Error>;
    async fn connect(&self, query: ConnectQuery) -> Result<DataConnectionIdWrapper, error::Error>;
    async fn disconnect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error>;
    async fn status(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionStatus, error::Error>;
    async fn redirect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error>;
    async fn event(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionEventEnum, error::Error>;
}
