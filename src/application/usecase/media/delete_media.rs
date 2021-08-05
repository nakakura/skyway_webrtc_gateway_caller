use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::ResponseMessage;
use crate::domain::media::service::MediaApi;
use crate::domain::media::value_object::MediaIdWrapper;
use crate::error;
use crate::prelude::ResponseMessageBodyEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteMediaService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
}

#[async_trait]
impl Service for DeleteMediaService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.delete_media(params).await?;
        Ok(ResponseMessage::Success(
            ResponseMessageBodyEnum::MediaContentDelete(MediaIdWrapper { media_id: param }),
        ))
    }
}

#[cfg(test)]
mod test_delete_media {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::di::MediaContentDeleteServiceContainer;
    use crate::domain::common::value_object::SerializableId;
    use crate::domain::media::service::MockMediaApi;
    use crate::domain::media::value_object::MediaId;
    use crate::error;
    use crate::prelude::ResponseMessageBodyEnum;

    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let media_id = MediaId::try_create("vi-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let expected = ResponseMessage::Success(ResponseMessageBodyEnum::MediaContentDelete(
            MediaIdWrapper {
                media_id: media_id.clone(),
            },
        ));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_delete_media().returning(move |_| {
            return Ok(media_id.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentDeleteServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
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

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let expected = serde_json::to_string(&error::Error::create_local_error("error")).unwrap();
        let expected = ResponseMessage::Error(expected);

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_delete_media()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentDeleteServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
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
