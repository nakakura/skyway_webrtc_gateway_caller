use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::{ReturnMessage, Service};
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::service::create_service;
use crate::domain::peer::value_object::PeerInfo;

#[cfg(test)]
use mockall_double::double;

pub(crate) const CREATE_PEER_COMMAND: &'static str = "PEER_CREATE";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerSuccessMessage {
    pub result: bool, // should be true
    pub command: String,
    pub params: PeerInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct ErrorMessage {
    pub result: bool, // should be false
    pub command: String,
    pub error_message: String,
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
    async fn execute_internal(&self, params: Value) -> Result<ReturnMessage, error::Error> {
        let peer_info = create_service::try_create(&self.repository, params).await?;
        let message_obj = CreatePeerSuccessMessage {
            result: true,
            command: CREATE_PEER_COMMAND.into(),
            params: peer_info.clone(),
        };
        Ok(ReturnMessage::PEER_CREATE(message_obj))
    }
}

#[async_trait]
impl Service for CreateService {
    fn command(&self) -> &'static str {
        return CREATE_PEER_COMMAND;
    }

    async fn execute(&self, params: Value) -> ReturnMessage {
        let result = self.execute_internal(params).await;
        self.create_return_message(result)
    }
}

#[cfg(test)]
mod test_create_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::PeerCreateServiceContainer;

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
        let message_obj = CreatePeerSuccessMessage {
            result: true,
            command: CREATE_PEER_COMMAND.into(),
            params: peer_info.clone(),
        };
        let expected = ReturnMessage::PEER_CREATE(message_obj);

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

        let expected = ReturnMessage::ERROR(ErrorMessage {
            result: false,
            command: CREATE_PEER_COMMAND.into(),
            error_message: format!("{:?}", error::Error::create_local_error("error")),
        });

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
