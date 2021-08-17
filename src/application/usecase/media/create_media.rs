use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::media::service::MediaApi;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateMediaService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
}

#[async_trait]
impl Service for CreateMediaService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.create_media(params).await?;
        Ok(MediaResponseMessageBodyEnum::ContentCreate(param).create_response_message())
    }
}

#[cfg(test)]
mod test_create_media {
    use std::sync::Mutex;

    use crate::error;
    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::MediaContentCreateServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableSocket;
    use crate::domain::webrtc::common::value_object::SocketInfo;
    use crate::domain::webrtc::media::service::MockMediaApi;
    use crate::domain::webrtc::media::value_object::MediaId;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let media_id = SocketInfo::<MediaId>::try_create(
            Some("vi-50a32bab-b3d9-4913-8e20-f79c90a6a211".into()),
            "127.0.0.1".into(),
            10000,
        )
        .unwrap();
        let expected =
            MediaResponseMessageBodyEnum::ContentCreate(media_id.clone()).create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_create_media().returning(move |_| {
            return Ok(media_id.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentCreateServiceContainer::builder()
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
        mock.expect_create_media()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentCreateServiceContainer::builder()
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
