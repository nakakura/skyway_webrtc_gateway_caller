use async_trait::async_trait;
use shaku::Interface;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::data::entity::{
    ConnectQuery, DataConnectionEventEnum, DataConnectionStatus, RedirectDataParams,
    RedirectDataResponse,
};
use crate::domain::webrtc::data::value_object::{DataConnectionId, DataId};
use crate::error;

#[cfg(test)]
use mockall::automock;

/// /data APIに対応する機能を定義する
#[cfg_attr(test, automock)]
#[async_trait]
pub trait DataRepository: Interface {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error>;
    async fn delete(&self, data_id: &DataId) -> Result<(), error::Error>;
    async fn connect(&self, query: ConnectQuery) -> Result<DataConnectionId, error::Error>;
    async fn disconnect(&self, data_connection_id: &DataConnectionId) -> Result<(), error::Error>;
    async fn status(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionStatus, error::Error>;
    async fn redirect(
        &self,
        data_conenction_id: &DataConnectionId,
        redirect_data_params: &RedirectDataParams,
    ) -> Result<RedirectDataResponse, error::Error>;
    async fn event(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionEventEnum, error::Error>;
}
