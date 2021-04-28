use serde::{Deserialize, Serialize};
use shaku::HasComponent;

use skyway_webrtc_gateway_api::error;

use super::create_peer::ErrorMessage;
use crate::di::PeerRepositoryContainer;
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::value_object::Peer;
use crate::domain::peer::value_object::PeerInfo;

#[cfg(test)]
use mockall_double::double;

pub(crate) const DELETE_PEER_COMMAND: &'static str = "DeletePeer";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
struct DeletePeerSuccessMessage {
    result: bool, // should be true
    command: &'static str,
    params: PeerInfo,
}

pub(crate) struct DeletePeer {}

impl DeletePeer {
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
        let repository: std::sync::Arc<dyn PeerRepository> = module.resolve();
        let peer = Peer::try_create_local(repository, message).await?;
        let result = peer.delete().await;

        let message_obj = DeletePeerSuccessMessage {
            result: true,
            command: DELETE_PEER_COMMAND,
            params: result.unwrap(),
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

        // deleteメソッドを成功させる正常なPeerのMockを作る
        let mut mock = Peer::default();
        mock.expect_delete().return_once(|| Ok(peer_info));

        // Peerの生成に成功するケース
        // 上で作ったMockを返す
        let ctx = Peer::try_create_local_context();
        ctx.expect().return_once(|_, _| Ok(mock));

        // execute
        let target = DeletePeer {};
        let result = target.execute("message").await;

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
        let ctx = Peer::try_create_local_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // execute
        let target = DeletePeer {};
        let result = target.execute("message").await;

        // clear the context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, expected);
    }
}
