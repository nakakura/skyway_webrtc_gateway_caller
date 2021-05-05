use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::peer::create::ErrorMessage;
use crate::application::usecase::service::{ReturnMessage, Service};
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::service::delete_service;
use crate::domain::peer::value_object::PeerInfo;

#[cfg(test)]
use mockall_double::double;

pub(crate) const DELETE_PEER_COMMAND: &'static str = "PEER_DELETE";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct DeletePeerSuccessMessage {
    result: bool, // should be true
    command: String,
    params: PeerInfo,
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepository>,
}

impl DeleteService {
    async fn execute_internal(&self, message: Value) -> Result<ReturnMessage, error::Error> {
        let peer_info = delete_service::try_delete(&self.repository, message).await?;

        let message_obj = DeletePeerSuccessMessage {
            result: true,
            command: DELETE_PEER_COMMAND.into(),
            params: peer_info,
        };
        Ok(ReturnMessage::PEER_DELETE(message_obj))
    }
}

#[async_trait]
impl Service for DeleteService {
    fn command(&self) -> &'static str {
        return DELETE_PEER_COMMAND;
    }

    async fn execute(&self, params: Value) -> ReturnMessage {
        let result = self.execute_internal(params).await;
        self.create_return_message(result)
    }
}

#[cfg(test)]
mod test_delete_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::PeerDeleteServiceContainer;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解の値を作成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let message_obj = DeletePeerSuccessMessage {
            result: true,
            command: DELETE_PEER_COMMAND.into(),
            params: peer_info.clone(),
        };
        let expected = ReturnMessage::PEER_DELETE(message_obj);

        // Peerの生成に成功するケース
        let ret_peer_info = peer_info.clone();
        let ctx = delete_service::try_delete_context();
        ctx.expect().return_once(|_, _| Ok(ret_peer_info));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerDeleteServiceContainer::builder().build();
        let delete_service: &dyn Service = module.resolve_ref();

        // execute
        let result = delete_service
            .execute(serde_json::to_value(&peer_info).unwrap())
            .await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解の値を作成
        let expected = ErrorMessage {
            result: false,
            command: DELETE_PEER_COMMAND.into(),
            error_message: format!("{:?}", error::Error::create_local_error("error")),
        };
        let expected = ReturnMessage::ERROR(expected);

        // Peerの生成に失敗するケース
        let ctx = delete_service::try_delete_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerDeleteServiceContainer::builder().build();
        let delete_service: &dyn Service = module.resolve_ref();

        // execute
        let result = delete_service.execute(serde_json::Value::Bool(true)).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }
}
