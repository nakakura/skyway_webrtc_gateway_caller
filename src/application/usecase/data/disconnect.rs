use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::{ErrorMessageRefactor, ResponseMessage, Service};
use crate::domain::data::service::DataApi;
use crate::ResponseMessageContent;
use skyway_webrtc_gateway_api::data::DataConnectionIdWrapper;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum DataDisconnectResponseMessage {
    Success(ResponseMessageContent<DataConnectionIdWrapper>),
    Error(ErrorMessageRefactor),
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DisconnectService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

impl DisconnectService {
    async fn execute_internal(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.disconnect(params).await?;
        let content = ResponseMessageContent::new(param);
        Ok(ResponseMessage::DataDisconnect(
            DataDisconnectResponseMessage::Success(content),
        ))
    }
}

#[async_trait]
impl Service for DisconnectService {
    fn command(&self) -> &'static str {
        return "";
    }

    async fn execute(&self, params: Value) -> ResponseMessage {
        let result = self.execute_internal(params).await;
        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ResponseMessage::DataDisconnect(DataDisconnectResponseMessage::Error(
                    ErrorMessageRefactor::new(message),
                ))
            }
        }
    }
}

#[cfg(test)]
mod test_create_data {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::error;

    use super::*;
    use crate::application::usecase::ErrorMessage;
    use crate::di::DataDisconnectServiceContainer;
    use crate::domain::data::service::MockDataApi;
    use crate::domain::data::value_object::{DataConnectionId, DataConnectionIdWrapper};

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
        let disconnect_service: &dyn Service = module.resolve_ref();

        // 引数を生成
        let message = format!(
            r#"{{
            "data_connection_id": "{}"
        }}"#,
            data_connection_id.as_str()
        );
        let message = serde_json::to_value(message).unwrap();

        //実行
        let result = disconnect_service.execute(message).await;

        // evaluate
        assert_eq!(serde_json::to_string(&result).unwrap(), "{\"is_success\":true,\"result\":{\"data_connection_id\":\"dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c\"}}");
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let err = error::Error::create_local_error("create error");
        let expected = ResponseMessage::DataDisconnect(DataDisconnectResponseMessage::Error(
            ErrorMessageRefactor::new(format!("{:?}", err)),
        ));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_disconnect()
            .returning(move |_| Err(error::Error::create_local_error("create error")));

        // Mockを埋め込んだServiceを生成
        let module = DataDisconnectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let connect_service: &dyn Service = module.resolve_ref();

        // execute
        let result = connect_service.execute(serde_json::Value::Null).await;

        // evaluate
        assert_eq!(result, expected);
    }
}
