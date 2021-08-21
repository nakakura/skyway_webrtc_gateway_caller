use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall_double::double;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::peer_refactor::repository::PeerRepositoryApiRefactor;
#[cfg_attr(test, double)]
use crate::domain::webrtc::peer_refactor::value_object::Peer;
use crate::domain::webrtc::peer_refactor::value_object::PeerInfo;
use crate::error;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct StatusService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepositoryApiRefactor>,
}

#[async_trait]
impl Service for StatusService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let params = serde_json::from_value::<PeerInfo>(params)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        let (_, status) = Peer::find(self.repository.clone(), params).await?;
        Ok(PeerResponseMessageBodyEnum::Status(status).create_response_message())
    }
}

#[cfg(test)]
mod test_create_peer {
    use super::*;
    use crate::di::PeerStatusServiceRefactorContainer;
    use crate::domain::webrtc::peer::value_object::PeerInfo;
    use crate::domain::webrtc::peer_refactor::value_object::PeerStatusMessage;

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // 正常終了するケースとして値を生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let expected = PeerResponseMessageBodyEnum::Status(
            PeerStatusMessage {
                peer_id: peer_info.peer_id().clone(),
                disconnected: false,
            }
            .clone(),
        )
        .create_response_message();

        // 正しくstatusを返すmockを作成
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                Some(Peer::default()),
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: false,
                },
            ))
        });

        // 実行時の引数(エンドユーザから与えられるはずの値)を生成
        let message = serde_json::to_string(&peer_info).unwrap();
        let message = serde_json::from_str::<Value>(&message).unwrap();

        // diでサービスを作成
        let module = PeerStatusServiceRefactorContainer::builder().build();
        let create_service: Arc<dyn Service> = module.resolve();

        // 実行
        let result = create_service.execute(message).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn invalid_json() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // ユーザがtokenを指定してこなかった場合
        let message = r#"{
            "peer_id": "peer_id"
        }"#;
        let message = serde_json::from_str::<Value>(message).unwrap();

        // diでサービスを作成
        let module = PeerStatusServiceRefactorContainer::builder().build();
        let create_service: Arc<dyn Service> = module.resolve();

        // 実行
        let result = create_service.execute(message).await;

        // evaluate
        if let Err(error::Error::SerdeError { error: _ }) = result {
            // JSONが間違っているので、ドメイン層の知識に従ってrejectされる
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn invalid_api() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // Peer::try_createが失敗するケース
        let ctx = Peer::try_create_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("peer生成失敗")));

        // 実行時の引数(エンドユーザから与えられるはずの値)を生成
        let message = serde_json::to_string(&peer_info).unwrap();
        let message = serde_json::from_str::<Value>(&message).unwrap();

        // diでサービスを作成
        let module = PeerStatusServiceRefactorContainer::builder().build();
        let create_service: Arc<dyn Service> = module.resolve();

        // 失敗するケースのmock
        let ctx = Peer::find_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("status api failed")));

        // 実行
        let result = create_service.execute(message).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        if let Err(error::Error::LocalError(message)) = result {
            assert_eq!(&message, "status api failed");
        } else {
            assert!(false);
        }
    }
}
