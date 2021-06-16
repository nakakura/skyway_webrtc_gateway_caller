use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::{ResponseMessage, Service};
use crate::domain::data::service::DataApi;
use skyway_webrtc_gateway_api::data::DataConnectionIdWrapper;

pub(crate) const CONNECT_COMMAND: &'static str = "DATA_CONNECT";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
pub struct DataConnectSuccessMessage {
    pub result: bool, // should be true
    pub command: String,
    pub params: DataConnectionIdWrapper,
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct ConnectService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

impl ConnectService {
    async fn execute_internal(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.connect(params).await?;
        Ok(ResponseMessage::DATA_CONNECT(DataConnectSuccessMessage {
            result: true,
            command: CONNECT_COMMAND.to_string(),
            params: param,
        }))
    }
}

#[async_trait]
impl Service for ConnectService {
    fn command(&self) -> &'static str {
        return CONNECT_COMMAND;
    }

    async fn execute(&self, params: Value) -> ResponseMessage {
        let param = self.execute_internal(params).await;
        self.create_return_message(param)
    }
}

#[cfg(test)]
mod test_create_data {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::error;

    use super::*;
    use crate::application::usecase::ErrorMessage;
    use crate::di::DataConnectServiceContainer;
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
        mock.expect_connect().returning(move |_| {
            return Ok(DataConnectionIdWrapper {
                data_connection_id: data_connection_id.clone(),
            });
        });

        // Mockを埋め込んだServiceを生成
        let module = DataConnectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let connect_service: &dyn Service = module.resolve_ref();

        // 引数を生成
        let message = r#"{{
            "peer_id": "peer_id",
            "token": "pt-9749250e-d157-4f80-9ee2-359ce8524308",
            "options": {{
                "metadata": "metadata",
                "serialization": "BINARY",
                "dcInit": {{
                    "ordered": true,
                    "maxPacketLifeTime": 0,
                    "maxRetransmits": 0,
                    "negotiated": true,
                    "id": 0
                }}
            }},
            "target_id": "ID_BAR",
            "params": {{
                "data_id": "da-50a32bab-b3d9-4913-8e20-f79c90a6a211"
            }},
            "redirect_params": {{
                "ip_v4": "127.0.0.1",
                "port": 10001
            }}
        }}"#;
        let message = serde_json::to_value(message).unwrap();

        //実行
        let result = connect_service.execute(message).await;

        // evaluate
        assert_eq!(serde_json::to_string(&result).unwrap(), "{\"result\":true,\"command\":\"DATA_CONNECT\",\"params\":{\"data_connection_id\":\"dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c\"}}");
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let err = error::Error::create_local_error("create error");
        let expected = ResponseMessage::ERROR(ErrorMessage {
            result: false,
            command: CONNECT_COMMAND.to_string(),
            error_message: format!("{:?}", err),
        });

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_connect()
            .returning(move |_| Err(error::Error::create_local_error("create error")));

        // Mockを埋め込んだServiceを生成
        let module = DataConnectServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let connect_service: &dyn Service = module.resolve_ref();

        // execute
        let result = connect_service.execute(serde_json::Value::Null).await;

        // evaluate
        assert_eq!(result, expected);
    }
}