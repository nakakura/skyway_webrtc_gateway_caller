use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{MediaResponse, ResponseResult};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::media::entity::MediaConnectionIdWrapper;
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct StatusService {
    #[shaku(inject)]
    repository: Arc<dyn MediaRepository>,
}

#[async_trait]
impl Service for StatusService {
    async fn execute(&self, params: Parameter) -> Result<ResponseResult, error::Error> {
        let media_connection_id = params
            .deserialize::<MediaConnectionIdWrapper>()?
            .media_connection_id;
        let status = self.repository.status(&media_connection_id).await?;
        Ok(MediaResponse::Status(status).create_response_message())
    }
}

#[cfg(test)]
mod test_create_media {
    use crate::di::MediaStatusServiceContainer;
    use crate::domain::webrtc::media::entity::{MediaConnectionIdWrapper, MediaConnectionStatus};
    use crate::domain::webrtc::media::repository::MockMediaRepository;
    use crate::domain::webrtc::media::value_object::MediaConnectionId;
    use crate::domain::webrtc::peer::value_object::PeerId;

    use super::*;

    #[tokio::test]
    async fn success() {
        // 期待値の生成
        let expected_status = MediaConnectionStatus {
            metadata: "metadata".to_string(),
            open: false,
            remote_id: PeerId::new("peer_id"),
            ssrc: None,
        };
        let expected = MediaResponse::Status(expected_status.clone()).create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_status().returning(move |_| {
            return Ok(expected_status.clone());
        });

        // Mockを埋め込んだStatusServiceを生成
        let module = MediaStatusServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let status_service: Arc<dyn Service> = module.resolve();

        // 実行に必要なパラメータの生成
        let param = MediaConnectionIdWrapper {
            media_connection_id: MediaConnectionId::try_create(
                "mc-50a32bab-b3d9-4913-8e20-f79c90a6a211",
            )
            .unwrap(),
        };

        // 実行
        let result = status_service
            .execute(Parameter(serde_json::to_value(param).unwrap()))
            .await
            .unwrap();

        // 実行に成功するので、statusが帰ってくる
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // Mockを埋め込んだStatusServiceを生成
        // 実行されないのでmockは初期化は不要
        let mock = MockMediaRepository::default();
        let module = MediaStatusServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let status_service: Arc<dyn Service> = module.resolve();

        // 異常なパラメータをつめて実行
        let result = status_service
            .execute(Parameter(serde_json::value::Value::Bool(true)))
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
