use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::domain::data::service::DataApi;
use crate::error;
use crate::prelude::ResponseMessageBodyEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DisconnectService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

#[async_trait]
impl Service for DisconnectService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.disconnect(params).await?;
        Ok(ResponseMessage::Success(ResponseMessageBodyEnum::Data(
            DataResponseMessageBodyEnum::Disconnect(param),
        )))
    }
}

#[cfg(test)]
mod test_create_data {
    use std::sync::Mutex;

    use crate::error;
    use once_cell::sync::Lazy;

    use crate::di::DataDisconnectServiceContainer;
    use crate::domain::data::service::MockDataApi;
    use crate::domain::data::value_object::{DataConnectionId, DataConnectionIdWrapper};

    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // CONNECTに成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        let ret_value = data_connection_id.clone();
        mock.expect_disconnect().returning(move |_| {
            return Ok(DataConnectionIdWrapper {
                data_connection_id: ret_value.clone(),
            });
        });

        // Mockを埋め込んだServiceを生成
        let module = DataDisconnectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let disconnect_service: Arc<dyn Service> = module.resolve();

        // 引数を生成
        let message = format!(
            r#"{{
            "data_connection_id": "{}"
        }}"#,
            data_connection_id.as_str()
        );
        let message = serde_json::to_value(message).unwrap();

        //実行
        let result =
            crate::application::usecase::service::execute_service(disconnect_service, message)
                .await;

        // evaluate
        assert_eq!(serde_json::to_string(&result).unwrap(), "{\"is_success\":true,\"result\":{\"type\":\"DATA\",\"command\":\"DISCONNECT\",\"data_connection_id\":\"dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c\"}}");
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
        mock.expect_disconnect()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだServiceを生成
        let module = DataDisconnectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let connect_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = crate::application::usecase::service::execute_service(
            connect_service,
            serde_json::Value::Null,
        )
        .await;

        // evaluate
        assert_eq!(result, expected);
    }
}
