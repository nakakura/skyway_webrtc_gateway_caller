use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall_double::double;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::webrtc::peer::service::create_service;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepository>,
}

#[async_trait]
impl Service for CreateService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let peer_info = create_service::try_create(&self.repository, params).await?;
        Ok(PeerResponseMessageBodyEnum::Create(peer_info).create_response_message())
    }
}

#[cfg(test)]
mod test_create_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::PeerCreateServiceContainer;
    use crate::domain::webrtc::peer::value_object::PeerInfo;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let message = r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#;
        let message = serde_json::from_str::<Value>(message).unwrap();

        // 正常終了するケースとして値を生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let expected =
            PeerResponseMessageBodyEnum::Create(peer_info.clone()).create_response_message();

        // 正しいPeerInfoを返す正常系動作
        let ret_peer_info = peer_info.clone();
        let ctx = create_service::try_create_context();
        ctx.expect().return_once(|_, _| Ok(ret_peer_info));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerCreateServiceContainer::builder().build();
        let create_service: Arc<dyn Service> = module.resolve();

        // execute
        let result =
            crate::application::usecase::service::execute_service(create_service, message).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let message = r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "turn": true
        }"#;
        let message = serde_json::from_str::<Value>(message).unwrap();

        let expected = serde_json::to_string(&error::Error::create_local_error("error")).unwrap();
        let expected = ResponseMessage::Error(expected);

        // Peerの生成に失敗するケース
        let ctx = create_service::try_create_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerCreateServiceContainer::builder().build();
        let create_service: Arc<dyn Service> = module.resolve();

        // execute
        let result =
            crate::application::usecase::service::execute_service(create_service, message).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }
}
