use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::data;
use skyway_webrtc_gateway_api::data::{DataConnectionStatus, RedirectDataParams};
use skyway_webrtc_gateway_api::prelude::PhantomId;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::data::service::DataApi;
use crate::domain::webrtc::data::value_object::{
    DataConnectionEventEnum, DataConnectionId, DataConnectionIdWrapper, DataId, DataIdWrapper,
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

    async fn delete(&self, data_id: DataId) -> Result<DataId, error::Error> {
        let _ = data::close_data_socket(&data_id).await?;
        Ok(data_id)
    }

    async fn connect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error> {
        use crate::domain::webrtc::data::value_object::ConnectQuery;
        let params = serde_json::from_value::<ConnectQuery>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        data::connect(params)
            .await
            .map(|id| DataConnectionIdWrapper {
                data_connection_id: id,
            })
    }

    async fn disconnect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error> {
        let data_connection_id = serde_json::from_value::<DataConnectionIdWrapper>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .data_connection_id;
        let _ = data::disconnect(&data_connection_id).await?;
        Ok(DataConnectionIdWrapper { data_connection_id })
    }

    async fn status(
        &self,
        data_connection_id: &DataConnectionId,
    ) -> Result<DataConnectionStatus, error::Error> {
        data::status(data_connection_id).await
    }

    async fn redirect(&self, params: Value) -> Result<DataConnectionIdWrapper, error::Error> {
        #[derive(Deserialize)]
        struct RedirectParams {
            pub data_connection_id: DataConnectionId,
            pub feed_params: Option<DataIdWrapper>,
            pub redirect_params: Option<SocketInfo<PhantomId>>,
        }
        let params = serde_json::from_value::<RedirectParams>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?;

        let redirect_data_params = RedirectDataParams {
            feed_params: params.feed_params,
            redirect_params: params.redirect_params,
        };
        let data_connection_id = params.data_connection_id;

        data::redirect(&data_connection_id, &redirect_data_params)
            .await
            .map(|_redirect| DataConnectionIdWrapper { data_connection_id })
    }

    async fn event(&self, params: Value) -> Result<DataConnectionEventEnum, error::Error> {
        let data_connection_id = serde_json::from_value::<DataConnectionIdWrapper>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .data_connection_id;
        data::event(&data_connection_id).await
    }
}
