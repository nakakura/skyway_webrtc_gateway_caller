use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::data;
use skyway_webrtc_gateway_api::error;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::service::DataApi;
use crate::domain::data::value_object::DataId;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = DataApi)]
pub(crate) struct DataApiImpl;

impl Default for DataApiImpl {
    fn default() -> Self {
        DataApiImpl {}
    }
}

// シンプルなのでテストはしていない
#[async_trait]
impl DataApi for DataApiImpl {
    async fn create(&self) -> Result<SocketInfo<DataId>, error::Error> {
        data::open_data_socket().await
    }

    async fn delete(&self, data_id: Value) -> Result<DataId, error::Error> {
        // FIXME
        unreachable!()
    }
}
