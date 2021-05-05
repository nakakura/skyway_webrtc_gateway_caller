use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;

use crate::application::usecase::service::{ReturnMessage, Service};
use crate::domain::peer::value_object::{Peer, PeerEventEnum, PeerInfo};

pub(crate) const PEER_EVENT_COMMAND: &'static str = "PEER_EVENT";

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
pub struct PeerEventMessage {
    result: bool, // should be true
    command: String,
    params: PeerEventEnum,
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct EventService {
    #[shaku(inject)]
    api: Arc<dyn Peer>,
}

impl EventService {
    async fn execute_internal(&self, message: Value) -> Result<ReturnMessage, error::Error> {
        let event = self.api.event(message).await?;
        let event_message = PeerEventMessage {
            result: true,
            command: PEER_EVENT_COMMAND.into(),
            params: event,
        };
        Ok(ReturnMessage::PEER_EVENT(event_message))
    }
}

#[async_trait]
impl Service for EventService {
    fn command(&self) -> &'static str {
        return PEER_EVENT_COMMAND;
    }

    async fn execute(&self, params: Value) -> ReturnMessage {
        let result = self.execute_internal(params).await;
        self.create_return_message(result)
    }
}

#[cfg(test)]
mod test_peer_event {
    use skyway_webrtc_gateway_api::peer::PeerCloseEvent;

    use super::*;
    use crate::application::usecase::peer::create::ErrorMessage;
    use crate::di::PeerEventServiceContainer;
    use crate::domain::peer::value_object::MockPeer;

    #[tokio::test]
    async fn success() {
        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let event = PeerEventEnum::CLOSE(PeerCloseEvent {
            params: peer_info.clone(),
        });

        // 期待値の生成
        let expected = ReturnMessage::PEER_EVENT(PeerEventMessage {
            result: true,
            command: PEER_EVENT_COMMAND.into(),
            params: event.clone(),
        });

        // CLOSEイベントを返すMockを作る
        let ret_event = event;
        let mut mock = MockPeer::default();
        mock.expect_event().return_once(move |_| Ok(ret_event));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn Peer>(Box::new(mock))
            .build();
        let event_service: &dyn Service = module.resolve_ref();

        // execute
        let result = event_service
            .execute(serde_json::to_value(&peer_info).unwrap())
            .await;

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 期待値の生成
        let expected = ReturnMessage::ERROR(ErrorMessage {
            result: false,
            command: PEER_EVENT_COMMAND.into(),
            error_message: format!("{:?}", error::Error::create_local_error("error")),
        });

        // CLOSEイベントを返すMockを作る
        let mut mock = MockPeer::default();
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn Peer>(Box::new(mock))
            .build();
        let event_service: &dyn Service = module.resolve_ref();

        // execute
        let result = event_service
            .execute(serde_json::to_value(&peer_info).unwrap())
            .await;

        // evaluate
        assert_eq!(result, expected);
    }
}
