use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::media::service::MediaApi;
use crate::domain::webrtc::media::value_object::{MediaConnection, MediaConnectionIdWrapper};
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct StatusService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
}

#[async_trait]
impl Service for StatusService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let media_connection_id = serde_json::from_value::<MediaConnectionIdWrapper>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .media_connection_id;
        let (_, status) = MediaConnection::find(self.api.clone(), media_connection_id).await?;
        Ok(MediaResponseMessageBodyEnum::Status(status).create_response_message())
    }
}

#[cfg(test)]
mod test_create_media {
    use super::*;
    use crate::di::MediaStatusServiceContainer;
    use crate::domain::webrtc::media::service::MockMediaApi;
    use crate::domain::webrtc::media::value_object::{MediaConnectionId, MediaConnectionStatus};
    use crate::domain::webrtc::peer::value_object::PeerId;

    #[tokio::test]
    async fn success() {
        // 期待値の生成
        let expected_status = MediaConnectionStatus {
            metadata: "metadata".to_string(),
            open: false,
            remote_id: PeerId::new("peer_id"),
            ssrc: None,
        };
        let expected =
            MediaResponseMessageBodyEnum::Status(expected_status.clone()).create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_status().returning(move |_| {
            return Ok(expected_status.clone());
        });

        // Mockを埋め込んだStatusServiceを生成
        let module = MediaStatusServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
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
            .execute(serde_json::to_value(param).unwrap())
            .await
            .unwrap();

        // 実行に成功するので、statusが帰ってくる
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // Mockを埋め込んだStatusServiceを生成
        // 実行されないのでmockは初期化は不要
        let mock = MockMediaApi::default();
        let module = MediaStatusServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let status_service: Arc<dyn Service> = module.resolve();

        // 異常なパラメータをつめて実行
        let result = status_service
            .execute(serde_json::value::Value::Bool(true))
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
