use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::error;
use tokio::sync::mpsc;

use crate::application::usecase::service::EventListener;
use crate::application::usecase::value_object::ResponseMessage;
use crate::di::ApplicationStateContainer;
use crate::domain::peer::value_object::{Peer, PeerEventEnum};
use crate::domain::utility::ApplicationState;
use crate::prelude::ResponseMessageBodyEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    api: Arc<dyn Peer>,
}

impl EventService {
    async fn execute_internal(&self, message: Value) -> Result<ResponseMessage, error::Error> {
        let event = self.api.event(message).await?;
        Ok(ResponseMessage::Success(
            ResponseMessageBodyEnum::PeerEvent(event),
        ))
    }
}

#[async_trait]
impl EventListener for EventService {
    async fn execute(
        &self,
        event_tx: mpsc::Sender<ResponseMessage>,
        params: Value,
    ) -> ResponseMessage {
        let module = ApplicationStateContainer::builder().build();
        let state: &dyn ApplicationState = module.resolve_ref();

        while state.is_running() {
            println!("state is running");
            let result = self.execute_internal(params.clone()).await;
            let flag = result.is_err();
            let message = match result {
                Ok(message) => message,
                Err(e) => {
                    let message = format!("{:?}", e);
                    ResponseMessage::Error(message)
                }
            };
            // send event
            let result = event_tx.send(message.clone()).await;

            // APIからerrorを受け取っていたら終了する
            if flag {
                return message;
            }
            // event_txへの送信がエラーなら終了する
            if let Err(e) = result {
                let message = format!("{:?}", e);
                return ResponseMessage::Error(message);
            }

            // close eventを受け取っていたら終了する
            if let ResponseMessage::Success(ResponseMessageBodyEnum::PeerEvent(
                ref peer_event_message,
            )) = message
            {
                if let PeerEventEnum::CLOSE(_) = &peer_event_message {
                    return message;
                }
            }
        }

        unreachable!();
    }
}

#[cfg(test)]
mod test_peer_event {
    use skyway_webrtc_gateway_api::data::{DataConnectionId, DataConnectionIdWrapper};
    use skyway_webrtc_gateway_api::peer::{PeerCloseEvent, PeerConnectionEvent};

    use crate::di::PeerEventServiceContainer;
    use crate::domain::peer::value_object::{MockPeer, PeerInfo};

    use super::*;

    #[tokio::test]
    async fn connect_and_close() {
        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let connect_event = PeerEventEnum::CONNECTION(PeerConnectionEvent {
            params: peer_info.clone(),
            data_params: DataConnectionIdWrapper {
                data_connection_id: DataConnectionId::try_create(
                    "dc-102127d9-30de-413b-93f7-41a33e39d82b",
                )
                .unwrap(),
            },
        });
        let close_event = PeerEventEnum::CLOSE(PeerCloseEvent {
            params: peer_info.clone(),
        });

        // 期待値の生成
        let expected_connect =
            ResponseMessage::Success(ResponseMessageBodyEnum::PeerEvent(connect_event.clone()));
        let expected_close =
            ResponseMessage::Success(ResponseMessageBodyEnum::PeerEvent(close_event.clone()));

        // 1回目はCONNECT、2回目はCLOSEイベントを返すMockを作る
        let mut counter = 0u8;
        let mut mock = MockPeer::default();
        mock.expect_event().returning(move |_| {
            counter += 1;
            if counter <= 1 {
                Ok(connect_event.clone())
            } else {
                Ok(close_event.clone())
            }
        });

        // Mockを埋め込んだEventServiceを生成
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn Peer>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // execute
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);
        let return_result = event_service
            .execute(event_tx, serde_json::to_value(&peer_info).unwrap())
            .await;

        // evaluate
        assert_eq!(return_result, expected_close);

        // eventが通知されていることを確認
        // 1つめはCONNECTイベント
        let result = event_rx.recv().await;
        if let Some(result_close_event) = result {
            assert_eq!(result_close_event, expected_connect);
        } else {
            assert!(false);
        }

        // 2つめはCLOSEイベント
        let result = event_rx.recv().await;
        if let Some(result_close_event) = result {
            assert_eq!(result_close_event, expected_close);
        } else {
            assert!(false);
        }

        // 3つ以上は来ない
        let result = event_rx.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn fail() {
        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 期待値の生成
        let err = format!("{:?}", error::Error::create_local_error("error"));
        let expected = ResponseMessage::Error(err);

        // CLOSEイベントを返すMockを作る
        let mut mock = MockPeer::default();
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn Peer>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // execute
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);
        let result = event_service
            .execute(event_tx, serde_json::to_value(&peer_info).unwrap())
            .await;

        // evaluate
        assert_eq!(result, expected);

        // eventが通知されていることを確認
        // ERRORを受信していることを確認
        let result = event_rx.recv().await.unwrap();
        assert_eq!(result, expected);

        // ERRORのあとは狩猟済みなのでイベントは来ない
        let result = event_rx.recv().await;
        assert!(result.is_none());
    }
}
