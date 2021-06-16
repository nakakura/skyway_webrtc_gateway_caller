use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::{
    ErrorMessageRefactor, ResponseMessage, ResponseMessageContent, Service,
};
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::service::delete_service;
use crate::domain::peer::value_object::PeerInfo;

#[cfg(test)]
use mockall_double::double;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PeerDeleteResponseMessage {
    Success(ResponseMessageContent<PeerInfo>),
    Error(ErrorMessageRefactor),
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
    async fn execute_internal(&self, message: Value) -> Result<ResponseMessage, error::Error> {
        let peer_info = delete_service::try_delete(&self.repository, message).await?;
        let content = ResponseMessageContent::new(peer_info);
        Ok(ResponseMessage::PeerDelete(
            PeerDeleteResponseMessage::Success(content),
        ))
    }
}

#[async_trait]
impl Service for DeleteService {
    async fn execute(&self, params: Value) -> ResponseMessage {
        let result = self.execute_internal(params).await;

        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ResponseMessage::PeerDelete(PeerDeleteResponseMessage::Error(
                    ErrorMessageRefactor::new(message),
                ))
            }
        }
    }
}

#[cfg(test)]
mod test_delete_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::application::usecase::ErrorMessage;
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
        let content = ResponseMessageContent::new(peer_info.clone());
        let expected = ResponseMessage::PeerDelete(PeerDeleteResponseMessage::Success(content));

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
        let expected = ResponseMessage::PeerDelete(PeerDeleteResponseMessage::Error(
            ErrorMessageRefactor::new(format!("{:?}", error::Error::create_local_error("error"))),
        ));

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
