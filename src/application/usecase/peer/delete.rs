use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::webrtc::peer::service::delete_service;
use crate::error;

#[cfg(test)]
use mockall_double::double;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepository>,
}

#[async_trait]
impl Service for DeleteService {
    async fn execute(&self, message: Value) -> Result<ResponseMessage, error::Error> {
        let peer_info = delete_service::try_delete(&self.repository, message).await?;
        Ok(PeerResponseMessageBodyEnum::Delete(peer_info).create_response_message())
    }
}

#[cfg(test)]
mod test_delete_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::PeerDeleteServiceContainer;
    use crate::domain::webrtc::peer::value_object::PeerInfo;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解の値を作成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let expected =
            PeerResponseMessageBodyEnum::Delete(peer_info.clone()).create_response_message();

        // Peerの生成に成功するケース
        let ret_peer_info = peer_info.clone();
        let ctx = delete_service::try_delete_context();
        ctx.expect().return_once(|_, _| Ok(ret_peer_info));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerDeleteServiceContainer::builder().build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = crate::application::usecase::service::execute_service(
            delete_service,
            serde_json::to_value(&peer_info).unwrap(),
        )
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
        let expected = serde_json::to_string(&error::Error::create_local_error("error")).unwrap();
        let expected = ResponseMessage::Error(expected);

        // Peerの生成に失敗するケース
        let ctx = delete_service::try_delete_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerDeleteServiceContainer::builder().build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = crate::application::usecase::service::execute_service(
            delete_service,
            serde_json::Value::Bool(true),
        )
        .await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }
}
