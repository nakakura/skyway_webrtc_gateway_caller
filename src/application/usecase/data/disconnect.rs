use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::data::entity::DataConnectionIdWrapper;
use crate::domain::webrtc::data::repository::DataRepository;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DisconnectService {
    #[shaku(inject)]
    repository: Arc<dyn DataRepository>,
}

#[async_trait]
impl Service for DisconnectService {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error> {
        let data_connection_id = params
            .deserialize::<DataConnectionIdWrapper>()?
            .data_connection_id;
        let _ = self.repository.disconnect(&data_connection_id).await?;
        Ok(
            DataResponseMessageBodyEnum::Disconnect(DataConnectionIdWrapper { data_connection_id })
                .create_response_message(),
        )
    }
}

#[cfg(test)]
mod test_create_data {
    use crate::di::DataDisconnectServiceContainer;
    use crate::domain::webrtc::data::repository::MockDataRepository;
    use crate::domain::webrtc::data::value_object::DataConnectionId;
    use crate::error;

    use super::*;

    #[tokio::test]
    async fn success() {
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 期待値を生成
        let wrapper = DataConnectionIdWrapper {
            data_connection_id: data_connection_id.clone(),
        };
        let expected = DataResponseMessageBodyEnum::Disconnect(wrapper).create_response_message();

        // CONNECTに成功する場合のMockを作成
        let mut mock = MockDataRepository::default();
        mock.expect_disconnect().returning(move |_| Ok(()));

        // Mockを埋め込んだServiceを生成
        let module = DataDisconnectServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let disconnect_service: Arc<dyn Service> = module.resolve();

        // 引数を生成
        let message = DataConnectionIdWrapper {
            data_connection_id: data_connection_id.clone(),
        };
        let message = Parameter(serde_json::to_value(message).unwrap());

        //実行
        let result = disconnect_service.execute(message).await.unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_params() {
        // このmockは呼ばれないので、初期化は必要ない
        let mock = MockDataRepository::default();

        // Mockを埋め込んだServiceを生成
        let module = DataDisconnectServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let connect_service: Arc<dyn Service> = module.resolve();

        // 適当な値で実行
        let result = connect_service
            .execute(Parameter(serde_json::Value::Bool(true)))
            .await;

        // evaluate
        if let Err(error::Error::SerdeError { error: _ }) = result {
            // JSONが間違っているので、ドメイン層の知識に従ってrejectされる
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
