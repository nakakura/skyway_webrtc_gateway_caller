use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::domain::data::service::DataApi;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct RedirectService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

#[async_trait]
impl Service for RedirectService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.redirect(params).await?;
        Ok(DataResponseMessageBodyEnum::Redirect(param).create_response_message())
    }
}

#[cfg(test)]
mod test_redirect_data {
    use std::sync::Mutex;

    use crate::error;
    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::DataRedirectServiceContainer;
    use crate::domain::data::service::MockDataApi;
    use crate::prelude::{DataConnectionId, DataConnectionIdWrapper};

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let expected = DataResponseMessageBodyEnum::Redirect(DataConnectionIdWrapper {
            data_connection_id: DataConnectionId::try_create(
                "dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c",
            )
            .unwrap(),
        })
        .create_response_message();

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
            return Ok(DataConnectionIdWrapper {
                data_connection_id: DataConnectionId::try_create(
                    "dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c",
                )
                .unwrap(),
            }
            .clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataRedirectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

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
        let expected = serde_json::to_string(&error::Error::create_local_error("error")).unwrap();
        let expected = ResponseMessage::Error(expected);

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_redirect()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = DataRedirectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

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
