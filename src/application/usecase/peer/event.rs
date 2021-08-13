use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::usecase::service::EventListener;
use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
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
    #[shaku(inject)]
    state: Arc<dyn ApplicationState>,
}

#[async_trait]
impl EventListener for EventService {
    async fn execute(
        &self,
        event_tx: mpsc::Sender<ResponseMessage>,
        params: Value,
    ) -> ResponseMessage {
        while self.state.is_running() {
            let event = self.api.event(params.clone()).await;
            match event {
                Ok(PeerEventEnum::CLOSE(ref _event)) => {
                    let message = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
                        PeerResponseMessageBodyEnum::Event(event.unwrap().clone()),
                    ));
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(PeerEventEnum::TIMEOUT) => {
                    // TIMEOUTはユーザに通知する必要がない
                }
                Ok(event) => {
                    let message = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
                        PeerResponseMessageBodyEnum::Event(event),
                    ));
                    let _ = event_tx.send(message.clone()).await;
                }
                Err(e) => {
                    let message = serde_json::to_string(&e).unwrap();
                    let message = ResponseMessage::Error(message);
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
            }
        }

        ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Event(PeerEventEnum::TIMEOUT),
        ))
    }
}

#[cfg(test)]
mod test_peer_event {
    use crate::di::PeerEventServiceContainer;
    use crate::domain::data::value_object::*;
    use crate::domain::peer::value_object::*;
    use crate::error;
    use crate::infra::utility::ApplicationStateAlwaysFalseImpl;

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
        let expected_connect = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Event(connect_event.clone()),
        ));
        let expected_close = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Event(close_event.clone()),
        ));

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

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // CLOSE Eventで抜けた場合はCLOSE Eventが帰ってくる
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
        let err = serde_json::to_string(&error::Error::create_local_error("error")).unwrap();
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

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // ERRORが発生してループを抜けたErrorが帰ってくる
        assert_eq!(result, expected);

        // eventが通知されていることを確認
        // ERRORを受信していることを確認
        let result = event_rx.recv().await.unwrap();
        assert_eq!(result, expected);

        // ERRORのあとは狩猟済みなのでイベントは来ない
        let result = event_rx.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn loop_exit() {
        // create parameter
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 期待値の生成
        let expected = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Event(PeerEventEnum::TIMEOUT),
        ));

        // CLOSEイベントを返すMockを作る
        let mut mock = MockPeer::default();
        mock.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = PeerEventServiceContainer::builder()
            .with_component_override::<dyn Peer>(Box::new(mock))
            .with_component_override::<dyn ApplicationState>(Box::new(
                ApplicationStateAlwaysFalseImpl {},
            ))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // execute
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // Application Stateがfalseを返すことによってループを抜けた場合は、TIMEOUTが帰ってくる
        let result = event_service
            .execute(event_tx, serde_json::to_value(&peer_info).unwrap())
            .await;
        assert_eq!(result, expected);

        // event発生前にApplicationStateによりloopを抜けている
        let result = event_rx.recv().await;
        assert_eq!(result, None);
    }
}
