use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall_double::double;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{PeerResponse, ResponseResult};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::peer::entity::CreatePeerParams;
use crate::domain::webrtc::peer::repository::PeerRepository;
#[cfg_attr(test, double)]
use crate::domain::webrtc::peer::service::create;
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
    async fn execute(&self, params: Parameter) -> Result<ResponseResult, error::Error> {
        // 汎用的なDTOオブジェクトであるParameterから必要な値を取り出せるかチェックするのはアプリケーション層の責務である
        let params = params.deserialize::<CreatePeerParams>()?;
        let peer_info = create::execute(self.repository.clone(), params).await?;
        Ok(PeerResponse::Create(peer_info).create_response_message())
    }
}

#[cfg(test)]
mod test_create_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::di::PeerCreateServiceContainer;
    use crate::domain::webrtc::peer::value_object::PeerInfo;

    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正常終了するケースとして値を生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let expected = PeerResponse::Create(peer_info.clone()).create_response_message();

        let ret_peer_info = peer_info.clone();
        let ctx = create::execute_context();
        ctx.expect()
            .return_once(move |_, _| Ok(ret_peer_info.clone()));

        // 実行時の引数(エンドユーザから与えられるはずの値)を生成
        let message = r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#;
        let message = serde_json::from_str::<Parameter>(message).unwrap();

        // diでサービスを作成
        let module = PeerCreateServiceContainer::builder().build();
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
        let _lock = LOCKER.lock();

        // ユーザがpeer_idを指定してこなかった場合
        let message = r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "turn": true
        }"#;
        let message = serde_json::from_str::<Parameter>(message).unwrap();

        // diでサービスを作成
        let module = PeerCreateServiceContainer::builder().build();
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
        let _lock = LOCKER.lock();

        // Peer::try_createが失敗するケース
        let ctx = create::execute_context();
        ctx.expect()
            .return_once(|_, _| Err(error::Error::create_local_error("peer生成失敗")));

        // 実行時の引数(エンドユーザから与えられるはずの値)を生成
        let message = r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#;
        let message = serde_json::from_str::<Parameter>(message).unwrap();

        // diでサービスを作成
        let module = PeerCreateServiceContainer::builder().build();
        let create_service: Arc<dyn Service> = module.resolve();

        // 実行
        let result = create_service.execute(message).await;

        // clear context
        ctx.checkpoint();

        // evaluate
        if let Err(error::Error::LocalError(message)) = result {
            assert_eq!(&message, "peer生成失敗");
        } else {
            assert!(false);
        }
    }
}
