use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{ResponseMessage, ResponseMessageBody};
use crate::domain::data::service::DataApi;
use crate::domain::data::value_object::DataIdWrapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum DataRedirectResponseMessage {
    Success(ResponseMessageBody<DataIdWrapper>),
    Error(ResponseMessageBody<String>),
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct RedirectService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

impl RedirectService {}

#[async_trait]
impl Service for RedirectService {
    fn create_error_message(&self, message: String) -> ResponseMessage {
        ResponseMessage::DataRedirect(DataRedirectResponseMessage::Error(
            ResponseMessageBody::new(message),
        ))
    }

    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.redirect(params).await?;
        let content = ResponseMessageBody::new(param);
        Ok(ResponseMessage::DataRedirect(
            DataRedirectResponseMessage::Success(content),
        ))
    }
}

#[cfg(test)]
mod test_redirect_data {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::error;

    use super::*;
    use crate::di::DataRedirectServiceContainer;
    use crate::domain::common::value_object::SerializableId;
    use crate::domain::data::service::MockDataApi;
    use crate::domain::data::value_object::DataId;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let data_id = DataId::try_create("da-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let expected = ResponseMessage::DataRedirect(DataRedirectResponseMessage::Success(
            ResponseMessageBody::new(DataIdWrapper {
                data_id: data_id.clone(),
            }),
        ));

        // API Callのためのパラメータを生成
        let parameter = r#"
        {
            "data_connection_id": "dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c",
            "feed_params": {
                "data_id": "da-50a32bab-b3d9-4913-8e20-f79c90a6a211"
            },
            "redirect_params": {
                "ip_v4": "127.0.0.1",
                "port": 10001
            }
        }"#;
        let parameter = serde_json::to_value(parameter).unwrap();

        // redirectに成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_redirect().returning(move |_| {
            return Ok(DataIdWrapper {
                data_id: data_id.clone(),
            }
            .clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataRedirectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result =
            crate::application::usecase::service::execute_service(create_service, parameter).await;

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let err = error::Error::create_local_error("create error");
        let expected = ResponseMessage::DataRedirect(DataRedirectResponseMessage::Error(
            ResponseMessageBody::new(format!("{:?}", err)),
        ));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_redirect()
            .returning(move |_| Err(error::Error::create_local_error("create error")));

        // Mockを埋め込んだEventServiceを生成
        let module = DataRedirectServiceContainer::builder()
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
