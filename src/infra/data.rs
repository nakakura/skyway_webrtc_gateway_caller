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
        use serde::Deserialize;
        #[derive(Deserialize, Debug)]
        struct Message {
            pub data_id: DataId,
        }
        let data_id = serde_json::from_value::<Message>(data_id)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .data_id;
        let _ = data::close_data_socket(&data_id).await?;
        Ok(data_id)
    }
}
