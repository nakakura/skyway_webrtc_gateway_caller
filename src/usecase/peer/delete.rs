use serde::{Deserialize, Serialize};
use shaku::HasComponent;

use skyway_webrtc_gateway_api::error;

use crate::di::PeerRepositoryContainer;
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::service::delete_service;
use crate::domain::peer::value_object::PeerInfo;
use crate::usecase::peer::create::ErrorMessage;

#[cfg(test)]
use mockall_double::double;

pub(crate) const DELETE_PEER_COMMAND: &'static str = "DeletePeer";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
struct DeletePeerSuccessMessage {
    result: bool, // should be true
    command: &'static str,
    params: PeerInfo,
}

pub(crate) struct DeleteService {}

impl DeleteService {
    pub async fn execute(&self, message: &str) -> String {
        match self.execute_internal(message).await {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                let err = ErrorMessage {
                    result: false,
                    command: DELETE_PEER_COMMAND,
                    error_message: message,
                };
                serde_json::to_string(&err).unwrap()
            }
        }
    }

    async fn execute_internal(&self, message: &str) -> Result<String, error::Error> {
        let module = PeerRepositoryContainer::builder().build();
        let repository: &dyn PeerRepository = module.resolve_ref();
        let peer_info = delete_service::try_delete(repository, message).await?;

        let message_obj = DeletePeerSuccessMessage {
            result: true,
            command: DELETE_PEER_COMMAND,
            params: peer_info,
        };
        Ok(serde_json::to_string(&message_obj)
            .map_err(|e| error::Error::SerdeError { error: e })?)
    }
}

#[cfg(test)]
mod test_delete_peer {
    use std::sync::Mutex;

    use super::*;
    use once_cell::sync::Lazy;

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
            command: DELETE_PEER_COMMAND,
            params: peer_info.clone(),
        };
        let expected = serde_json::to_string(&message_obj).unwrap();

        // Peerの生成に成功するケース
        let ctx = delete_service::try_delete_context();
        ctx.expect().return_once(|_, _| Ok(peer_info));

        // execute
        let target = DeleteService {};
        // FIXME: invalid parameter
        let result = target.execute("should be valid json").await;

        // clear the context
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
            command: DELETE_PEER_COMMAND,
            error_message: format!("{:?}", error::Error::create_local_error("error")),
        };
        let expected = serde_json::to_string(&expected).unwrap();

        // Peerの生成に失敗するケース
        let ctx = delete_service::try_delete_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // execute
        let target = DeleteService {};
        // FIXME: invalid parameter
        let result = target.execute("should be valid json").await;

        // clear the context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }
}
