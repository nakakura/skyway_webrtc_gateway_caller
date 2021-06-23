use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{ResponseMessage, ResponseMessageBody};
use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::service::DataApi;
use crate::domain::data::value_object::DataId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum DataCreateResponseMessage {
    Success(ResponseMessageBody<SocketInfo<DataId>>),
    Error(ResponseMessageBody<String>),
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

impl CreateService {}

#[async_trait]
impl Service for CreateService {
    fn create_error_message(&self, message: String) -> ResponseMessage {
        ResponseMessage::DataCreate(DataCreateResponseMessage::Error(ResponseMessageBody::new(
            message,
        )))
    }

    async fn execute(&self, _params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.create().await?;
        let content = ResponseMessageBody::new(param);
        Ok(ResponseMessage::DataCreate(
            DataCreateResponseMessage::Success(content),
        ))
    }
}

#[cfg(test)]
mod test_create_data {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::error;

    use super::*;
    use crate::di::DataCreateServiceContainer;
    use crate::domain::common::value_object::SerializableSocket;
    use crate::domain::data::service::MockDataApi;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let data_id = SocketInfo::<DataId>::try_create(
            Some("da-50a32bab-b3d9-4913-8e20-f79c90a6a211".into()),
            "127.0.0.1".into(),
            10000,
        )
        .unwrap();
        let expected = ResponseMessage::DataCreate(DataCreateResponseMessage::Success(
            ResponseMessageBody::new(data_id.clone()),
        ));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_create().returning(move || {
            return Ok(data_id.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataCreateServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result = crate::application::usecase::service::execute_service(
            create_service,
            serde_json::Value::Bool(true),
        )
        .await;

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let err = error::Error::create_local_error("create error");
        let expected = ResponseMessage::DataCreate(DataCreateResponseMessage::Error(
            ResponseMessageBody::new(format!("{:?}", err)),
        ));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_create()
            .returning(move || Err(error::Error::create_local_error("create error")));

        // Mockを埋め込んだEventServiceを生成
        let module = DataCreateServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result = crate::application::usecase::service::execute_service(
            create_service,
            serde_json::Value::Bool(true),
        )
        .await;

        // evaluate
        assert_eq!(result, expected);
    }
}
