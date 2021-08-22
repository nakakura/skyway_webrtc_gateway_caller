use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::data::service::DataApi;
use crate::domain::webrtc::data::value_object::{DataConnection, DataConnectionIdWrapper};
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct StatusService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

impl StatusService {}

#[async_trait]
impl Service for StatusService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let data_connection_id = serde_json::from_value::<DataConnectionIdWrapper>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .data_connection_id;
        let (_connection, status) =
            DataConnection::find(self.api.clone(), data_connection_id).await?;
        Ok(DataResponseMessageBodyEnum::Status(status).create_response_message())
    }
}

#[cfg(test)]
mod test_create_data {
    use super::*;
    use crate::di::DataStatusServiceContainer;
    use crate::domain::webrtc::data::service::MockDataApi;
    use crate::domain::webrtc::data::value_object::DataConnectionStatus;
    use skyway_webrtc_gateway_api::data::DataConnectionId;

    #[tokio::test]
    async fn success() {
        let status = DataConnectionStatus {
            remote_id: "remote_id".to_string(),
            buffersize: 0,
            label: "label".to_string(),
            metadata: "meta".to_string(),
            open: true,
            reliable: false,
            serialization: "BINARY".to_string(),
            r#type: "DATA".to_string(),
        };
        // 期待値を生成
        let expected =
            DataResponseMessageBodyEnum::Status(status.clone()).create_response_message();

        // statusの取得に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_status().returning(move |_| Ok(status.clone()));

        // Mockを埋め込んだEventServiceを生成
        let module = DataStatusServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let status_service: Arc<dyn Service> = module.resolve();

        // 実行用のパラメータ生成
        let data_connection_id_wrapper = DataConnectionIdWrapper {
            data_connection_id: DataConnectionId::try_create(
                "dc-50a32bab-b3d9-4913-8e20-f79c90a6a211",
            )
            .unwrap(),
        };
        let param = serde_json::to_value(data_connection_id_wrapper).unwrap();

        // 実行
        let result = status_service.execute(param).await.unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // このmockは呼ばれることはないので、初期化は必要ない
        let mock = MockDataApi::default();

        // Mockを埋め込んだEventServiceを生成
        let module = DataStatusServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let status_service: Arc<dyn Service> = module.resolve();

        // 適当なパラメータで実行
        let result = status_service.execute(serde_json::Value::Bool(true)).await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
