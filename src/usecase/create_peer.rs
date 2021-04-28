use serde::{Deserialize, Serialize};
use shaku::HasComponent;

use skyway_webrtc_gateway_api::error;

use crate::di::PeerRepositoryContainer;
use crate::domain::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::peer::value_object::Peer;
use crate::domain::peer::value_object::PeerInfo;

#[cfg(test)]
use mockall_double::double;

pub(crate) const CREATE_PEER_COMMAND: &'static str = "CreatePeer";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
struct CreatePeerSuccessMessage {
    result: bool, // should be true
    command: &'static str,
    params: PeerInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub(crate) struct ErrorMessage {
    pub result: bool, // should be false
    pub command: &'static str,
    pub error_message: String,
}

pub(crate) struct CreatePeer;

impl CreatePeer {
    pub async fn execute(&self, message: &str) -> String {
        match self.execute_internal(message).await {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                let err = ErrorMessage {
                    result: false,
                    command: CREATE_PEER_COMMAND,
                    error_message: message,
                };
                serde_json::to_string(&err).unwrap()
            }
        }
    }

    async fn execute_internal(&self, message: &str) -> Result<String, error::Error> {
        let module = PeerRepositoryContainer::builder().build();
        let repository: std::sync::Arc<dyn PeerRepository> = module.resolve();
        let peer = Peer::try_create(repository, message).await?;
        let peer_info = peer.peer_info();
        let message_obj = CreatePeerSuccessMessage {
            result: true,
            command: CREATE_PEER_COMMAND,
            params: peer_info.clone(),
        };
        Ok(serde_json::to_string(&message_obj)
            .map_err(|e| error::Error::SerdeError { error: e })?)
    }
}

#[cfg(test)]
mod test_create_peer {
    use std::sync::Mutex;

    use super::*;
    use once_cell::sync::Lazy;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正常終了するケースとして値を生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let message_obj = CreatePeerSuccessMessage {
            result: true,
            command: CREATE_PEER_COMMAND,
            params: peer_info.clone(),
        };
        let expected = serde_json::to_string(&message_obj).unwrap();

        // 正しいPeerInfoを返す正常なPeerのMockを作る
        let mut mock = Peer::default();
        mock.expect_peer_info().return_const(peer_info);

        // 上で作ったPeerの生成に成功するケース
        let ctx = Peer::try_create_context();
        ctx.expect().return_once(|_, _| Ok(mock));

        // execute
        let target = CreatePeer {};
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

        let expected = ErrorMessage {
            result: false,
            command: CREATE_PEER_COMMAND,
            error_message: format!("{:?}", error::Error::create_local_error("error")),
        };

        // Peerの生成に失敗するケース
        let ctx = Peer::try_create_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("error")));

        // execute
        let target = CreatePeer {};
        let result = target.execute("message").await;

        // clear the context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result, serde_json::to_string(&expected).unwrap());
    }
}
