use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::data::entity::{
    DataConnectionIdWrapper, RedirectDataParams, RedirectParams,
};
use crate::domain::webrtc::data::repository::DataRepository;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct RedirectService {
    #[shaku(inject)]
    repository: Arc<dyn DataRepository>,
}

#[async_trait]
impl Service for RedirectService {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error> {
        let params = params.deserialize::<RedirectParams>()?;
        let data_connection_id = params.data_connection_id;
        let redirect_data_params = RedirectDataParams {
            feed_params: params.feed_params,
            redirect_params: params.redirect_params,
        };
        let _ = self
            .repository
            .redirect(&data_connection_id, &redirect_data_params)
            .await?;
        let wrapper = DataConnectionIdWrapper {
            data_connection_id: data_connection_id,
        };

        Ok(DataResponseMessageBodyEnum::Redirect(wrapper).create_response_message())
    }
}

#[cfg(test)]
mod test_redirect_data {
    use crate::di::DataRedirectServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableId;
    use crate::domain::webrtc::data::entity::{
        DataConnectionId, DataConnectionIdWrapper, RedirectDataResponse,
    };
    use crate::domain::webrtc::data::repository::MockDataRepository;
    use crate::domain::webrtc::data::value_object::DataId;
    use crate::error;

    use super::*;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let expected = DataResponseMessageBodyEnum::Redirect(DataConnectionIdWrapper {
            data_connection_id: DataConnectionId::try_create(
                "dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c",
            )
            .unwrap(),
        })
        .create_response_message();

        // redirectに成功する場合のMockを作成
        let mut mock = MockDataRepository::default();
        mock.expect_redirect().returning(move |_, _| {
            Ok(RedirectDataResponse {
                command_type: "RESPONSE".to_string(),
                data_id: DataId::try_create("da-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap(),
            })
        });

        // API Callのためのパラメータを生成
        let param = RedirectParams {
            data_connection_id: DataConnectionId::try_create(
                "dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c",
            )
            .unwrap(),
            feed_params: None,
            redirect_params: None,
        };
        let param = serde_json::to_value(param).unwrap();

        // Mockを埋め込んだEventServiceを生成
        let module = DataRedirectServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let redirect_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = redirect_service.execute(Parameter(param)).await.unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_params() {
        // このMockは呼ばれないので、初期化は必要ない
        let mock = MockDataRepository::default();

        // Mockを埋め込んだEventServiceを生成
        let module = DataRedirectServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let redirect_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = redirect_service
            .execute(Parameter(serde_json::Value::Bool(true)))
            .await;

        // evaluate
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
