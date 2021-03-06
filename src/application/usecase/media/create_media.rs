use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{MediaResponse, ResponseResult};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::error;

// エンドユーザから渡されるJSONのparamsフィールドを構造化するためのStruct
#[derive(Serialize, Deserialize)]
struct IsVideo {
    is_video: bool,
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateMediaService {
    #[shaku(inject)]
    repository: Arc<dyn MediaRepository>,
}

#[async_trait]
impl Service for CreateMediaService {
    async fn execute(&self, params: Parameter) -> Result<ResponseResult, error::Error> {
        let is_video = params.deserialize::<IsVideo>()?.is_video;
        let socket = self.repository.create_media(is_video).await?;
        Ok(MediaResponse::ContentCreate(socket).create_response_message())
    }
}

#[cfg(test)]
mod test_create_media {
    use crate::di::MediaContentCreateServiceContainer;
    use crate::domain::webrtc::common::value_object::{SerializableSocket, SocketInfo};
    use crate::domain::webrtc::media::repository::MockMediaRepository;
    use crate::domain::webrtc::media::value_object::MediaId;
    use crate::error;

    use super::*;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let media_socket = SocketInfo::<MediaId>::try_create(
            Some("vi-50a32bab-b3d9-4913-8e20-f79c90a6a211".into()),
            "127.0.0.1".into(),
            10000,
        )
        .unwrap();
        let expected = MediaResponse::ContentCreate(media_socket.clone()).create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_create_media().returning(move |_| {
            return Ok(media_socket.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentCreateServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

        // execute
        let param = IsVideo { is_video: true };
        let result = create_service
            .execute(Parameter(serde_json::to_value(&param).unwrap()))
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_create_media()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaContentCreateServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = create_service
            .execute(Parameter(serde_json::Value::String("foo".into())))
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
