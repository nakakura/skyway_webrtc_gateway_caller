use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::peer::repository::PeerRepositoryApiRefactor;
#[cfg_attr(test, double)]
use crate::domain::webrtc::peer::value_object::Peer;
use crate::{error, PeerInfo};

#[cfg(test)]
use mockall_double::double;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct DeleteService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepositoryApiRefactor>,
}

#[async_trait]
impl Service for DeleteService {
    async fn execute(&self, param: Value) -> Result<ResponseMessage, error::Error> {
        let peer_info: PeerInfo =
            serde_json::from_value(param).map_err(|e| error::Error::SerdeError { error: e })?;
        let (peer_opt, _) = Peer::find(self.repository.clone(), peer_info).await?;
        if let Some(peer) = peer_opt {
            let result = peer.try_delete().await?;
            Ok(PeerResponseMessageBodyEnum::Delete(result).create_response_message())
        } else {
            Err(error::Error::create_local_error(
                "Peer has been already deleted.",
            ))
        }
    }
}

#[cfg(test)]
mod test_delete_peer {
    use super::*;
    use crate::di::PeerDeleteServiceRefactorContainer;
    use crate::domain::webrtc::peer::value_object::PeerInfo;
    use crate::domain::webrtc::peer::value_object::PeerStatusMessage;
    use crate::error;

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // 削除対象の情報を定義
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        // サービスに与えるパラメータ化
        let param = serde_json::to_value(&peer_info).unwrap();

        // 待値を生成
        let expected =
            PeerResponseMessageBodyEnum::Delete(peer_info.clone()).create_response_message();

        // deleteが成功するかのように振る舞うMockを作成
        let ret_value = peer_info.clone();
        let mut peer_mock = Peer::default();
        peer_mock
            .expect_try_delete()
            .returning(move || Ok(ret_value.clone()));

        // 正しくstatusを返すようMockを設定
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                Some(peer_mock),
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: false,
                },
            ))
        });

        // diでサービスを作成
        let module = PeerDeleteServiceRefactorContainer::builder().build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // 実行
        let result = delete_service.execute(param).await.unwrap();

        // clear context
        ctx.checkpoint();

        // 成功するケースなので、Deleteメッセージが帰る
        assert_eq!(result, expected);
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

        // Mockを埋め込んだEventServiceを生成
        let module = PeerDeleteServiceRefactorContainer::builder().build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // execute
        let result = delete_service.execute(message).await;

        // JSONのパースエラーが帰ってくる
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn already_released() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // 削除対象の情報を定義
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        // サービスに与えるパラメータ化
        let param = serde_json::to_value(&peer_info).unwrap();

        // deleteが成功するかのように振る舞うMockを作成
        let ret_value = peer_info.clone();
        let mut peer_mock = Peer::default();
        peer_mock
            .expect_try_delete()
            .returning(move || Ok(ret_value.clone()));

        // 正しくstatusを返すが、Peer Objectが削除済みのケース
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                None,
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: true,
                },
            ))
        });

        // diでサービスを作成
        let module = PeerDeleteServiceRefactorContainer::builder().build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // 実行
        let result = delete_service.execute(param).await;

        // clear context
        ctx.checkpoint();

        // 削除済みの場合エラーが帰る
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "Peer has been already deleted.")
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn api_failed() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // 削除対象の情報を定義
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        // サービスに与えるパラメータ化
        let param = serde_json::to_value(&peer_info).unwrap();

        // deleteが成功するかのように振る舞うMockを作成
        let mut peer_mock = Peer::default();
        peer_mock
            .expect_try_delete()
            .returning(move || Err(error::Error::create_local_error("try_delete method failed")));

        // 正しくstatusを返すようMockを設定
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                Some(peer_mock),
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: false,
                },
            ))
        });

        // diでサービスを作成
        let module = PeerDeleteServiceRefactorContainer::builder().build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // 実行
        let result = delete_service.execute(param).await;

        // clear context
        ctx.checkpoint();

        // 削除済みの場合エラーが帰る
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "try_delete method failed")
        } else {
            assert!(false);
        }
    }
}
