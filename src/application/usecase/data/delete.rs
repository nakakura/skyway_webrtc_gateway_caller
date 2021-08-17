use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::data::service::DataApi;
use crate::error;
use crate::prelude::DataIdWrapper;

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
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.delete(params).await?;
        Ok(
            DataResponseMessageBodyEnum::Delete(DataIdWrapper { data_id: param })
                .create_response_message(),
        )
    }
}

#[cfg(test)]
mod test_create_data {
    use std::sync::Mutex;

    use crate::error;
    use once_cell::sync::Lazy;
    use serde::Deserialize;

    use super::*;
    use crate::di::DataDeleteServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableId;
    use crate::domain::webrtc::data::service::MockDataApi;
    use crate::domain::webrtc::data::value_object::DataId;
    use crate::prelude::DataIdWrapper;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        let data_id_str = "da-50a32bab-b3d9-4913-8e20-f79c90a6a211";

        // 期待値を生成
        let expected = DataResponseMessageBodyEnum::Delete(DataIdWrapper {
            data_id: DataId::try_create(data_id_str).unwrap(),
        })
        .create_response_message();

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
        let delete_service: Arc<dyn Service> = module.resolve();

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
            "{\"reason\":\"JsonError\",\"message\":\"missing field `data_id`\"}".into(),
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
        let delete_service: Arc<dyn Service> = module.resolve();

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
