use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{PeerResponseMessageBodyEnum, ResponseMessage};
use crate::application::usecase::service::Service;
use crate::domain::webrtc::peer::repository::PeerRepository;
use crate::domain::webrtc::peer::value_object::PeerInfo;
use crate::error;

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
    async fn execute(&self, param: Parameter) -> Result<ResponseMessage, error::Error> {
        // 汎用的なDTOオブジェクトであるParameterから必要な値を取り出せるかチェックするのはアプリケーション層の責務である
        let peer_info = param.deserialize::<PeerInfo>()?;
        let _ = self.repository.delete(&peer_info).await?;
        // APIは削除するのみでpeer_infoを返さないが、削除に成功した場合は、ユーザの不利便性のためにpeer_infoを返す
        Ok(PeerResponseMessageBodyEnum::Delete(peer_info).create_response_message())
    }
}

#[cfg(test)]
mod test_delete_peer {
    use super::*;
    use crate::di::PeerDeleteServiceContainer;
    use crate::domain::webrtc::peer::repository::MockPeerRepository;
    use crate::error;

    #[tokio::test]
    async fn success() {
        // 削除対象の情報を定義
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 待値を生成
        let expected =
            PeerResponseMessageBodyEnum::Delete(peer_info.clone()).create_response_message();

        // 削除に成功するケースのmockを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_delete().returning(|_| Ok(()));

        // mockを埋め込んだサービスを作成
        let module = PeerDeleteServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // サービスに与えるパラメータ化
        let param = Parameter(serde_json::to_value(&peer_info).unwrap());
        // 実行
        let result = delete_service.execute(param).await.unwrap();

        // 成功するケースなので、Deleteメッセージが帰る
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn invalid_json() {
        // パラメータチェックの段階で終了されてしまい、呼ばれることのないmock
        let mut mock = MockPeerRepository::default();
        mock.expect_delete().returning(|_| unreachable!());

        // mockを埋め込んだサービスを作成
        let module = PeerDeleteServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // ユーザがtokenを指定してこなかった場合
        let message = r#"{
            "peer_id": "peer_id"
        }"#;
        let message = serde_json::from_str::<Parameter>(message).unwrap();
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
    async fn api_failed() {
        // 削除対象の情報を定義
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // エラーを返すMockを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_delete().returning(move |_| {
            Err(error::Error::create_local_error(
                "peer delete method failed",
            ))
        });

        // mockを埋め込んだサービスを作成
        let module = PeerDeleteServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let delete_service: Arc<dyn Service> = module.resolve();

        // サービスに与えるパラメータ化
        let param = Parameter(serde_json::to_value(&peer_info).unwrap());
        // 実行
        let result = delete_service.execute(param).await;

        // 削除済みの場合エラーが帰る
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "peer delete method failed")
        } else {
            assert!(false);
        }
    }
}
