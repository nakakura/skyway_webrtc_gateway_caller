use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::media::entity::MediaIdWrapper;
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteMediaService {
    #[shaku(inject)]
    repository: Arc<dyn MediaRepository>,
}

#[async_trait]
impl Service for DeleteMediaService {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error> {
        let media_id = params.deserialize::<MediaIdWrapper>()?.media_id;
        let _ = self.repository.delete_media(&media_id).await?;
        Ok(
            MediaResponseMessageBodyEnum::ContentDelete(MediaIdWrapper { media_id })
                .create_response_message(),
        )
    }
}

#[cfg(test)]
mod test_delete_media {
    use crate::di::MediaContentDeleteServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableId;
    use crate::domain::webrtc::media::repository::MockMediaRepository;
    use crate::domain::webrtc::media::value_object::MediaId;
    use crate::error;

    use super::*;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let media_id = MediaId::try_create("vi-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let expected = MediaResponseMessageBodyEnum::ContentDelete(MediaIdWrapper {
            media_id: media_id.clone(),
        })
        .create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_delete_media().returning(move |_| {
            return Ok(());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentDeleteServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let media_id = MediaIdWrapper {
            media_id: media_id.clone(),
        };
        let result = delete_service
            .execute(Parameter(serde_json::to_value(&media_id).unwrap()))
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_delete_media()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentDeleteServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = delete_service
            .execute(Parameter(serde_json::Value::Bool(true)))
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
