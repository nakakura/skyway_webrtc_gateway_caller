use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::media::service::MediaApi;
use crate::domain::webrtc::media::value_object::{
    CallQuery, MediaConnection, MediaConnectionIdWrapper,
};
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CallService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
}

#[async_trait]
impl Service for CallService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let call_query = serde_json::from_value::<CallQuery>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        let result = MediaConnection::try_create(self.api.clone(), call_query).await?;
        let wrapper = MediaConnectionIdWrapper {
            media_connection_id: result.params.media_connection_id,
        };
        Ok(MediaResponseMessageBodyEnum::Call(wrapper).create_response_message())
    }
}

#[cfg(test)]
mod test_create_media {
    use super::*;
    use crate::di::MediaCallServiceContainer;
    use crate::domain::webrtc::media::service::MockMediaApi;
    use crate::domain::webrtc::media::value_object::{
        CallResponse, MediaConnectionId, MediaConnectionIdWrapper,
    };
    use crate::domain::webrtc::peer::value_object::{PeerId, Token};

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let media_connection_id =
            MediaConnectionId::try_create("mc-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let expected = MediaResponseMessageBodyEnum::Call(MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        })
        .create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_call().returning(move |_query| {
            return Ok(CallResponse {
                command_type: "CALL".to_string(),
                params: MediaConnectionIdWrapper {
                    media_connection_id: media_connection_id.clone(),
                },
            });
        });

        // Mockを埋め込んだCallServiceを生成
        let module = MediaCallServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let call_service: Arc<dyn Service> = module.resolve();

        // 実行のためのパラメータを生成
        let call_query = CallQuery {
            peer_id: PeerId::new("peer_id"),
            token: Token::try_create("pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap(),
            target_id: PeerId::new("target_id"),
            constraints: None,
            redirect_params: None,
        };

        // execute
        let result = call_service
            .execute(serde_json::to_value(call_query).unwrap())
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_param() {
        // Mockを埋め込んだCallServiceを生成
        // このテストではMockは呼ばれないので、初期化は不要
        let mock = MockMediaApi::default();
        let module = MediaCallServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let call_service: Arc<dyn Service> = module.resolve();

        // 適当な値を与えて実行
        let result = call_service
            .execute(serde_json::to_value(serde_json::Value::Bool(true)).unwrap())
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
