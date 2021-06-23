use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::ResponseMessage;
use crate::domain::data::service::DataApi;
use crate::prelude::ResponseMessageBodyEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
}

#[async_trait]
impl Service for DeleteService {
    fn create_error_message(&self, message: String) -> ResponseMessage {
        ResponseMessage::Error(message)
    }

    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.delete(params).await?;
        Ok(ResponseMessage::Success(
            ResponseMessageBodyEnum::DataDelete(param),
        ))
    }
}

#[cfg(test)]
mod test_create_data {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use serde::Deserialize;
    use skyway_webrtc_gateway_api::error;

    use super::*;
    use crate::di::DataDeleteServiceContainer;
    use crate::domain::common::value_object::SerializableId;
    use crate::domain::data::service::MockDataApi;
    use crate::domain::data::value_object::DataId;
    use crate::prelude::ResponseMessageBodyEnum;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        let data_id_str = "da-50a32bab-b3d9-4913-8e20-f79c90a6a211";

        // 期待値を生成
        let expected = ResponseMessage::Success(ResponseMessageBodyEnum::DataDelete(
            DataId::try_create(data_id_str).unwrap(),
        ));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_delete().returning(move |message| {
            // 削除に成功した場合、削除対象のDataIdが帰る
            #[derive(Deserialize, Debug)]
            struct Message {
                pub data_id: DataId,
            }
            let result = serde_json::from_value::<Message>(message).unwrap();
            return Ok(result.data_id);
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataDeleteServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let delete_service: &dyn Service = module.resolve_ref();

        // execute
        let message = format!(
            r#"{{
                "data_id": "{}"
            }}"#,
            data_id_str
        );
        let message = serde_json::from_str::<Value>(&message).unwrap();
        let result =
            crate::application::usecase::service::execute_service(delete_service, message).await;

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_json() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        let data_id_str = "da-50a32bab-b3d9-4913-8e20-f79c90a6a211";

        // 期待値を生成
        let expected = ResponseMessage::Error(
            "SerdeError { error: Error(\"missing field `data_id`\", line: 0, column: 0) }"
                .to_string(),
        );

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataApi::default();
        mock.expect_delete().returning(move |message| {
            // 削除に成功した場合、削除対象のDataIdが帰る
            #[derive(Deserialize, Debug)]
            struct Message {
                pub data_id: DataId,
            }
            serde_json::from_value::<Message>(message)
                .map(|data| data.data_id)
                .map_err(|e| error::Error::SerdeError { error: e })
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataDeleteServiceContainer::builder()
            .with_component_override::<dyn DataApi>(Box::new(mock))
            .build();
        let delete_service: &dyn Service = module.resolve_ref();

        // execute
        let message = format!(
            r#"{{
                "data_iddd": "{}"
            }}"#,
            data_id_str
        );
        let message = serde_json::from_str::<Value>(&message).unwrap();
        let result =
            crate::application::usecase::service::execute_service(delete_service, message).await;

        // evaluate
        assert_eq!(result, expected);
    }
}
