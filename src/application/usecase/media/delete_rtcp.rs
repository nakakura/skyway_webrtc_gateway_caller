use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::domain::webrtc::media::value_object::{RtcpIdWrapper, RtcpSocket};
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteRtcpService {
    #[shaku(inject)]
    api: Arc<dyn MediaRepository>,
}

#[async_trait]
impl Service for DeleteRtcpService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let rtcp_id = serde_json::from_value::<RtcpIdWrapper>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?
            .rtcp_id;

        let _ = RtcpSocket::try_delete(self.api.clone(), &rtcp_id).await?;
        Ok(
            MediaResponseMessageBodyEnum::RtcpDelete(RtcpIdWrapper { rtcp_id })
                .create_response_message(),
        )
    }
}

#[cfg(test)]
mod test_delete_media {
    use super::*;
    use crate::di::MediaRtcpDeleteServiceContainer;
    use crate::domain::webrtc::common::value_object::SerializableId;
    use crate::domain::webrtc::media::repository::MockMediaRepository;
    use crate::domain::webrtc::media::value_object::RtcpId;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let rtcp_id = RtcpId::try_create("rc-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let expected = MediaResponseMessageBodyEnum::RtcpDelete(RtcpIdWrapper {
            rtcp_id: rtcp_id.clone(),
        })
        .create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_delete_rtcp().returning(move |_| Ok(()));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaRtcpDeleteServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let param = serde_json::to_value(RtcpIdWrapper {
            rtcp_id: rtcp_id.clone(),
        })
        .unwrap();
        let result = delete_service.execute(param).await.unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_delete_rtcp()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaRtcpDeleteServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = delete_service.execute(serde_json::Value::Bool(true)).await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
