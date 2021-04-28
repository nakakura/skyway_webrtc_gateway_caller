use serde::{Deserialize, Serialize};
use skyway_webrtc_gateway_api::error;

use crate::domain::peer::value_object::{Peer, PeerEventEnum, PeerInfo};

use crate::usecase::peer::create::ErrorMessage;

pub(crate) const PEER_EVENT_COMMAND: &'static str = "PEER_EVENT";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PeerEventMessage {
    result: bool, // should be true
    command: &'static str,
    params: PeerEventEnum,
}

pub(crate) struct EventService {}

impl EventService {
    pub async fn execute(&self, message: &str, api: &dyn Peer) -> String {
        match self.execute_internal(message, api).await {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                let err = ErrorMessage {
                    result: false,
                    command: PEER_EVENT_COMMAND,
                    error_message: message,
                };
                serde_json::to_string(&err).unwrap()
            }
        }
    }

    async fn execute_internal(
        &self,
        message: &str,
        api: &dyn Peer,
    ) -> Result<String, error::Error> {
        let event = api.event(message).await?;
        let error_message = PeerEventMessage {
            result: true,
            command: PEER_EVENT_COMMAND,
            params: event,
        };
        Ok(serde_json::to_string(&error_message)
            .map_err(|e| error::Error::SerdeError { error: e })?)
    }
}

#[cfg(test)]
mod test_peer_event {
    use std::sync::Mutex;

    use skyway_webrtc_gateway_api::peer::PeerCloseEvent;

    use super::*;
    use crate::domain::peer::value_object::MockPeer;
    use crate::usecase::peer::create::ErrorMessage;
    use once_cell::sync::Lazy;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        let _ = LOCKER.lock();

        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let event = PeerEventEnum::CLOSE(PeerCloseEvent {
            params: peer_info.clone(),
        });

        // CLOSEイベントを返すMockを作る
        let ret_event = event.clone();
        let mut mock = MockPeer::default();
        mock.expect_event().return_once(move |_| Ok(ret_event));

        // 期待値の生成
        let expected = PeerEventMessage {
            result: true,
            command: PEER_EVENT_COMMAND,
            params: event,
        };
        let expected = serde_json::to_string(&expected).unwrap();

        // execute
        let event = EventService {};
        let result = event
            .execute(&serde_json::to_string(&peer_info).unwrap(), &mock)
            .await;

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        let _ = LOCKER.lock();

        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // CLOSEイベントを返すMockを作る
        let mut mock = MockPeer::default();
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // 期待値の生成
        let expected = ErrorMessage {
            result: false,
            command: PEER_EVENT_COMMAND,
            error_message: format!("{:?}", error::Error::create_local_error("error")),
        };
        let expected = serde_json::to_string(&expected).unwrap();

        // execute
        let event = EventService {};
        let result = event
            .execute(&serde_json::to_string(&peer_info).unwrap(), &mock)
            .await;

        // evaluate
        assert_eq!(result, expected);
    }
}
