use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::media::service::MediaApi;
use crate::domain::webrtc::media::value_object::RtcpSocket;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateRtcpService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
}

#[async_trait]
impl Service for CreateRtcpService {
    async fn execute(&self, _params: Value) -> Result<ResponseMessage, error::Error> {
        let socket = RtcpSocket::try_create(self.api.clone()).await?;
        Ok(MediaResponseMessageBodyEnum::RtcpCreate(socket).create_response_message())
    }
}

#[cfg(test)]
mod test_create_rtcp {
    use super::*;
    use crate::di::MediaRtcpCreateServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableSocket;
    use crate::domain::webrtc::common::value_object::SocketInfo;
    use crate::domain::webrtc::media::service::MockMediaApi;
    use crate::domain::webrtc::media::value_object::RtcpId;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let rtcp_id = SocketInfo::<RtcpId>::try_create(
            Some("rc-50a32bab-b3d9-4913-8e20-f79c90a6a211".into()),
            "127.0.0.1".into(),
            10000,
        )
        .unwrap();
        let expected = MediaResponseMessageBodyEnum::RtcpCreate(RtcpSocket(rtcp_id.clone()))
            .create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_create_rtcp().returning(move || {
            return Ok(rtcp_id.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaRtcpCreateServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = create_service
            .execute(serde_json::Value::Bool(true))
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }
}
