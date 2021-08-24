use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::data::repository::DataRepository;
use crate::domain::webrtc::data::value_object::{DataIdWrapper, DataSocket};
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteService {
    #[shaku(inject)]
    api: Arc<dyn DataRepository>,
}

#[async_trait]
impl Service for DeleteService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let data_id = serde_json::from_value::<DataIdWrapper>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .data_id;

        let _ = DataSocket::try_delete(self.api.clone(), &data_id).await?;
        Ok(
            DataResponseMessageBodyEnum::Delete(DataIdWrapper { data_id: data_id })
                .create_response_message(),
        )
    }
}

#[cfg(test)]
mod test_create_data {
    use super::*;
    use crate::di::DataDeleteServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableId;
    use crate::domain::webrtc::data::repository::MockDataRepository;
    use crate::domain::webrtc::data::value_object::DataId;

    #[tokio::test]
    async fn success() {
        let data_id_str = "da-50a32bab-b3d9-4913-8e20-f79c90a6a211";

        // 期待値を生成
        let expected = DataResponseMessageBodyEnum::Delete(DataIdWrapper {
            data_id: DataId::try_create(data_id_str).unwrap(),
        })
        .create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataRepository::default();
        mock.expect_delete().returning(move |_data_id| {
            // 削除に成功した場合、削除対象のDataIdが帰る
            return Ok(());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = DataDeleteServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let message = DataIdWrapper {
            data_id: DataId::try_create(data_id_str).unwrap(),
        };
        let result = delete_service
            .execute(serde_json::to_value(message).unwrap())
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_json() {
        let data_id_str = "da-50a32bab-b3d9-4913-8e20-f79c90a6a211";

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockDataRepository::default();
        mock.expect_delete().returning(move |_data_id| Ok(()));

        // Mockを埋め込んだEventServiceを生成
        let module = DataDeleteServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
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
        let result = delete_service.execute(message).await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
