use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall_double::double;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::{
    ErrorMessageRefactor, ResponseMessage, ResponseMessageContent, Service,
};
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::service::create_service;
use crate::domain::peer::value_object::PeerInfo;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PeerCreateResponseMessage {
    Success(ResponseMessageContent<PeerInfo>),
    Error(ErrorMessageRefactor),
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct CreateService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepository>,
}

impl CreateService {
    async fn execute_internal(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let peer_info = create_service::try_create(&self.repository, params).await?;
        let content = ResponseMessageContent::new(peer_info);
        Ok(ResponseMessage::PeerCreate(
            PeerCreateResponseMessage::Success(content),
        ))
    }
}

#[async_trait]
impl Service for CreateService {
    fn command(&self) -> &'static str {
        return "";
    }

    async fn execute(&self, params: Value) -> ResponseMessage {
        let result = self.execute_internal(params).await;

        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ResponseMessage::PeerCreate(PeerCreateResponseMessage::Error(
                    ErrorMessageRefactor::new(message),
                ))
            }
        }
    }
}

#[cfg(test)]
mod test_create_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::application::usecase::ErrorMessage;
    use crate::di::PeerCreateServiceContainer;

    use super::*;
    use std::net::Shutdown::Read;

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
        let message_obj = ResponseMessageContent::new(peer_info.clone());
        let expected = ResponseMessage::PeerCreate(PeerCreateResponseMessage::Success(message_obj));

        // 正しいPeerInfoを返す正常系動作
        let ret_peer_info = peer_info.clone();
        let ctx = create_service::try_create_context();
        ctx.expect().return_once(|_, _| Ok(ret_peer_info));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerCreateServiceContainer::builder().build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result = create_service.execute(message).await;

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

        let expected =
            ErrorMessageRefactor::new(format!("{:?}", error::Error::create_local_error("error")));
        let expected = ResponseMessage::PeerCreate(PeerCreateResponseMessage::Error(expected));

        // Peerの生成に失敗するケース
        let ctx = create_service::try_create_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerCreateServiceContainer::builder().build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result = create_service.execute(message).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }
}
