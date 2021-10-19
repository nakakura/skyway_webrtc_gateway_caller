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
pub(crate) struct StatusService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepository>,
}

#[async_trait]
impl Service for StatusService {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error> {
        let peer_info = params.deserialize::<PeerInfo>()?;
        let status = self.repository.status(&peer_info).await?;
        Ok(PeerResponseMessageBodyEnum::Status(status).create_response_message())
    }
}

#[cfg(test)]
mod test_create_peer {
    use super::*;
    use crate::di::PeerStatusServiceContainer;
    use crate::domain::webrtc::peer::entity::PeerStatusMessage;
    use crate::domain::webrtc::peer::repository::MockPeerRepository;

    #[tokio::test]
    async fn success() {
        // 正常終了するケースとして値を生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let status = PeerStatusMessage {
            peer_id: peer_info.peer_id().clone(),
            disconnected: false,
        };
        let expected =
            PeerResponseMessageBodyEnum::Status(status.clone()).create_response_message();

        // 削除に成功するケースのmockを作成
        let result_value = status.clone();
        let mut mock = MockPeerRepository::default();
        mock.expect_status()
            .returning(move |_| Ok(result_value.clone()));

        // mockを埋め込んだサービスを作成
        let module = PeerStatusServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let status_service: Arc<dyn Service> = module.resolve();

        // 実行時の引数(エンドユーザから与えられるはずの値)を生成
        let message = serde_json::to_string(&peer_info).unwrap();
        let message = serde_json::from_str::<Parameter>(&message).unwrap();
        // 実行
        let result = status_service.execute(message).await;

        // evaluate
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn invalid_json() {
        // 呼ばれないことを確認するため、読んだらクラッシュするmockを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_status().returning(move |_| {
            assert!(false);
            unreachable!();
        });

        // diでサービスを作成
        let module = PeerStatusServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

        // ユーザがtokenを指定してこなかった場合
        let message = r#"{
            "peer_id": "peer_id"
        }"#;
        let message = serde_json::from_str::<Parameter>(message).unwrap();

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
        // エラーを返すmockを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_status()
            .return_once(|_| Err(error::Error::create_local_error("event api error(500)")));

        // diでサービスを作成
        let module = PeerStatusServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let create_service: Arc<dyn Service> = module.resolve();

        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 実行時の引数(エンドユーザから与えられるはずの値)を生成
        let message = serde_json::to_string(&peer_info).unwrap();
        let message = serde_json::from_str::<Parameter>(&message).unwrap();
        // 実行
        let result = create_service.execute(message).await;

        // evaluate
        if let Err(error::Error::LocalError(message)) = result {
            assert_eq!(&message, "event api error(500)");
        } else {
            assert!(false);
        }
    }
}
