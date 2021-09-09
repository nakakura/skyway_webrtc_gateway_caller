use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::data::entity::DataSocket;
use crate::domain::webrtc::data::repository::DataRepository;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateService {
    #[shaku(inject)]
    api: Arc<dyn DataRepository>,
}

impl CreateService {}

#[async_trait]
impl Service for CreateService {
    async fn execute(&self, _params: Parameter) -> Result<ResponseMessage, error::Error> {
        let data_sock = DataSocket::try_create(self.api.clone()).await?;
        Ok(DataResponseMessageBodyEnum::Create(data_sock).create_response_message())
    }
}

#[cfg(test)]
mod test_create_data {
    use crate::di::DataCreateServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableSocket;
    use crate::domain::webrtc::common::value_object::SocketInfo;
    use crate::domain::webrtc::data::repository::MockDataRepository;
    use crate::domain::webrtc::data::value_object::DataId;

    use super::*;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let data_id = SocketInfo::<DataId>::try_create(
            Some("da-50a32bab-b3d9-4913-8e20-f79c90a6a211".into()),
            "127.0.0.1".into(),
            10000,
        )
        .unwrap();
        let expected = DataResponseMessageBodyEnum::Create(DataSocket(data_id.clone()))
            .create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataRepository::default();
        mock.expect_create().returning(move || {
            return Ok(data_id.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataCreateServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

        // execute
        // 引数は利用しないので何でも良い
        let result = create_service
            .execute(Parameter(serde_json::Value::Bool(true)))
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }
}
