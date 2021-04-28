use skyway_webrtc_gateway_api::error;

use super::value_object::{CreatePeerParams, PeerInfo};
use crate::domain::peer::repository::PeerRepository;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub(crate) mod create_service {
    use super::*;

    pub(crate) async fn try_create(
        repository: &dyn PeerRepository,
        message: &str,
    ) -> Result<PeerInfo, error::Error> {
        // この位置でドメイン層の知識としてJSONの値をチェックする
        let params = serde_json::from_str::<CreatePeerParams>(message)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        repository.register(params).await
    }
}

#[cfg(test)]
mod test_peer_create {
    use super::*;
    use crate::domain::peer::repository::MockPeerRepository;

    fn create_valid_json_message() -> &'static str {
        r#"{
            "base_url": "http://localhost:8000",
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#
    }

    #[tokio::test]
    async fn success() {
        // create parameter
        let message = create_valid_json_message();

        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_register().returning(|_| {
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308")
        });

        // execute
        let result = create_service::try_create(&mock, message).await;

        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // evaluate
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn parameter_wrong() {
        // create parameter
        let invalid_message = r#"{
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "peer_id",
            "turn": true
        }"#;

        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_register().returning(|_| {
            unreachable!();
        });

        // execute
        let result = create_service::try_create(&mock, invalid_message).await;

        // evaluate
        if let Err(error::Error::SerdeError { error: e }) = result {
            let message = format!("{:?}", e);
            assert_eq!(
                message.as_str(),
                "Error(\"missing field `base_url`\", line: 6, column: 9)"
            );
        } else {
            unreachable!();
        }
    }

    #[tokio::test]
    async fn register_failed() {
        // create parameter
        let message = create_valid_json_message();

        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_register()
            .returning(|_| Err(error::Error::create_local_error("error")));

        // execute
        let result = create_service::try_create(&mock, message).await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "error");
        } else {
            unreachable!();
        }
    }
}

#[cfg_attr(test, automock)]
pub(crate) mod delete_service {
    use super::*;

    pub(crate) async fn try_delete(
        repository: &dyn PeerRepository,
        message: &str,
    ) -> Result<PeerInfo, error::Error> {
        // この位置でドメイン層の知識としてJSONの値をチェックする
        let peer_info = serde_json::from_str::<PeerInfo>(message)
            .map_err(|e| error::Error::SerdeError { error: e })?;

        let _ = repository.erase(&peer_info).await?;
        Ok(peer_info)
    }
}

#[cfg(test)]
mod test_peer_delete {
    use super::*;
    use crate::domain::peer::repository::MockPeerRepository;

    fn create_peer_info() -> PeerInfo {
        PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap()
    }

    fn create_message() -> String {
        serde_json::to_string(&create_peer_info()).unwrap()
    }

    #[tokio::test]
    async fn success() {
        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_erase().returning(|_| Ok(()));

        // execute
        let result = delete_service::try_delete(&mock, &create_message()).await;

        let expected = create_peer_info();

        // evaluate
        assert_eq!(result.unwrap(), expected);
    }

    #[tokio::test]
    async fn fail() {
        // setup mock
        let mut mock = MockPeerRepository::default();
        mock.expect_erase()
            .returning(|_| Err(error::Error::create_local_error("error")));

        // execute
        let result = delete_service::try_delete(&mock, &create_message()).await;

        // evaluate
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e.as_str(), "error");
        } else {
            unreachable!();
        }
    }
}
