use async_trait::async_trait;
use shaku::*;
use skyway_webrtc_gateway_api::data;
use skyway_webrtc_gateway_api::data::{DataConnectionStatus, RedirectDataParams};

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::data::service::DataApi;
use crate::domain::webrtc::data::value_object::{
    ConnectQuery, DataConnectionEventEnum, DataConnectionId, DataConnectionIdWrapper, DataId,
    RedirectDataResponse,
};
use crate::error;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = DataApi)]
pub(crate) struct DataApiImpl;

impl Default for DataApiImpl {
    fn default() -> Self {
        DataApiImpl {}
    }
}

// FIXME: シンプルなので単体テストはしていない。結合試験のみ
#[async_trait]
impl DataApi for DataApiImpl {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error> {
        data::open_data_socket().await
    }

    async fn delete(&self, data_id: &DataId) -> Result<(), error::Error> {
        data::close_data_socket(data_id).await
    }

    async fn connect(&self, query: ConnectQuery) -> Result<DataConnectionIdWrapper, error::Error> {
        data::connect(query)
            .await
            .map(|id| DataConnectionIdWrapper {
                data_connection_id: id,
            })
    }

    async fn disconnect(&self, data_connection_id: &DataConnectionId) -> Result<(), error::Error> {
        data::disconnect(&data_connection_id).await
    }

    async fn status(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionStatus, error::Error> {
        data::status(data_connection_id).await
    }

    async fn redirect(
        &self,
        data_connection_id: &DataConnectionId,
        redirect_data_params: &RedirectDataParams,
    ) -> Result<RedirectDataResponse, error::Error> {
        data::redirect(data_connection_id, redirect_data_params).await
    }

    async fn event(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionEventEnum, error::Error> {
        data::event(&data_connection_id).await
    }
}
