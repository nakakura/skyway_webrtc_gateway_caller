use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::usecase::service::EventListener;
use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
use crate::domain::state::ApplicationState;
use crate::domain::webrtc::peer::repository::PeerRepositoryApi;
#[cfg_attr(test, double)]
use crate::domain::webrtc::peer::value_object::Peer;
use crate::domain::webrtc::peer::value_object::{PeerEventEnum, PeerInfo};
use crate::error;

#[cfg(test)]
use mockall_double::double;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    api: Arc<dyn PeerRepositoryApi>,
    #[shaku(inject)]
    state: Arc<dyn ApplicationState>,
}

impl EventService {
    async fn listen_event(
        &self,
        peer: Peer,
        event_tx: mpsc::Sender<ResponseMessage>,
    ) -> ResponseMessage {
        while self.state.is_running() {
            let event = peer.try_event().await;
            match event {
                Ok(PeerEventEnum::CLOSE(event)) => {
                    let message =
                        PeerResponseMessageBodyEnum::Event(PeerEventEnum::CLOSE(event).clone())
                            .create_response_message();
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(PeerEventEnum::TIMEOUT) => {
                    // TIMEOUTはユーザに通知する必要がない
                }
                Ok(event) => {
                    let message =
                        PeerResponseMessageBodyEnum::Event(event).create_response_message();
                    let _ = event_tx.send(message.clone()).await;
                }
                Err(e) => {
                    let message = format!("error in EventService for Peer {:?}", e);
                    let message = ResponseMessage::Error(message);
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
            }
        }
        PeerResponseMessageBodyEnum::Event(PeerEventEnum::TIMEOUT).create_response_message()
    }
}

#[async_trait]
impl EventListener for EventService {
    async fn execute(
        &self,
        event_tx: mpsc::Sender<ResponseMessage>,
        params: Value,
    ) -> ResponseMessage {
        // peer_infoのvalidation
        let peer_info = serde_json::from_value::<PeerInfo>(params.clone())
            .map_err(|e| error::Error::SerdeError { error: e });
        if peer_info.is_err() {
            let message = format!(
                "Error in EventListener for Peer. invalid peer_info: {:?}",
                params
            );
            return ResponseMessage::Error(message);
        }
        let peer_info = peer_info.unwrap();

        match Peer::find(self.api.clone(), peer_info.clone()).await {
            Err(e) => {
                let message = format!("No Such Peer Object {:?}, returns error {:?}", peer_info, e);
                return ResponseMessage::Error(message);
            }
            Ok((_, status)) if status.disconnected => {
                let message = format!("Peer has been already deleted {:?}", peer_info);
                return ResponseMessage::Error(message);
            }
            Ok((peer, _)) => {
                return self.listen_event(peer, event_tx).await;
            }
        }
    }
}

#[cfg(test)]
mod test_peer_event {
    use std::sync::Mutex;

    use super::*;
    use crate::di::PeerEventServiceContainer;
    use crate::domain::webrtc::data::value_object::*;
    use crate::domain::webrtc::peer::value_object::{
        PeerCloseEvent, PeerConnectionEvent, PeerStatusMessage,
    };
    use crate::error;
    use crate::infra::state::ApplicationStateAlwaysFalseImpl;

    // 成功する場合
    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // いくつかのイベントを取得した後、CLOSEが発火すると監視終了
        // このテストでは、CONNECT, TIMEOUT, CLOSEの順に受信するものとする

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // イベントの作成
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
        // CONNECTのイベント
        let expected_connect =
            PeerResponseMessageBodyEnum::Event(connect_event.clone()).create_response_message();
        // CLOSEのイベント
        let expected_close =
            PeerResponseMessageBodyEnum::Event(close_event.clone()).create_response_message();

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // event_serviceを生成
        let module = &PeerEventServiceContainer::builder().build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 3つのイベントを返すmockを作成
        let counter = Mutex::new(0u8);
        let mut peer_mock = Peer::default();
        peer_mock.expect_try_event().returning(move || {
            let mut counter_ref = counter.lock().unwrap();
            *counter_ref += 1;
            match *counter_ref {
                1 => {
                    return Ok(connect_event.clone());
                }
                2 => {
                    return Ok(PeerEventEnum::TIMEOUT);
                }
                _ => {
                    return Ok(close_event.clone());
                }
            }
        });

        // 正しくstatusを返すようMockを設定
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                peer_mock,
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: false,
                },
            ))
        });

        // execute
        let result = event_service
            .execute(event_tx, serde_json::to_value(&peer_info).unwrap())
            .await;

        // clear context
        ctx.checkpoint();

        // CLOSEが発火してループを抜けた場合、最後はCLOSEを受信する
        assert_eq!(result, expected_close);

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

        // 3つ以上は来ない(TIMEOUTは受信しない)
        let result = event_rx.recv().await;
        assert!(result.is_none());
    }

    // eventはcloseが発火するか、stateがfalseを返すまで繰り返される
    // このケースは最初からstateがfalseを返すのでイベントを取得しに行かないパターン
    #[tokio::test]
    async fn exit_due_to_state() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (event_tx, _) = mpsc::channel::<ResponseMessage>(10);

        // このmockは呼ばれないので、初期化しないでよい
        let peer_mock = Peer::default();
        // 正しくstatusを返すようMockを設定
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                peer_mock,
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: false,
                },
            ))
        });

        // EventServiceを生成
        // stateは必ずfalseを返すモックを挿入
        let module = &PeerEventServiceContainer::builder()
            .with_component_override::<dyn ApplicationState>(Box::new(
                ApplicationStateAlwaysFalseImpl {},
            ))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // execute
        let result = event_service
            .execute(event_tx, serde_json::to_value(&peer_info).unwrap())
            .await;

        // clear context
        ctx.checkpoint();

        // stateによりイベントループを抜けた場合、最後はTIMEOUTを返す
        assert_eq!(
            result,
            PeerResponseMessageBodyEnum::Event(PeerEventEnum::TIMEOUT).create_response_message()
        );
    }

    // エンドユーザから与えられたjsonが間違っており、イベントを取得できない場合
    #[tokio::test]
    async fn invalid_json() {
        // event_serviceの引数は、JSON化されたPeerInfoとevent senderであるが、なぜかbool値が入ってきた
        let param = serde_json::Value::Bool(true);
        let (event_tx, _) = mpsc::channel::<ResponseMessage>(10);

        // event_serviceを生成
        let module = &PeerEventServiceContainer::builder().build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // execute
        let result = event_service.execute(event_tx, param).await;

        if let ResponseMessage::Error(message) = result {
            assert_eq!(
                &message,
                "Error in EventListener for Peer. invalid peer_info: Bool(true)"
            );
        } else {
            assert!(false);
        }
    }

    // APIがエラーを帰す場合
    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = crate::application::usecase::peer::PEER_FIND_MOCK_LOCKER.lock();

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (event_tx, _) = mpsc::channel::<ResponseMessage>(10);

        // event_serviceを生成
        let module = &PeerEventServiceContainer::builder().build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // errorを返すmockを作成
        let mut peer_mock = Peer::default();
        peer_mock
            .expect_try_event()
            .returning(move || return Err(error::Error::create_local_error("try_event error")));

        // 正しくstatusを返すようMockを設定
        let ctx = Peer::find_context();
        ctx.expect().return_once(|_, peer_info| {
            Ok((
                peer_mock,
                PeerStatusMessage {
                    peer_id: peer_info.peer_id().clone(),
                    disconnected: false,
                },
            ))
        });

        // execute
        let result = event_service
            .execute(event_tx, serde_json::to_value(&peer_info).unwrap())
            .await;

        // clear context
        ctx.checkpoint();

        // errorが帰ってくる
        if let ResponseMessage::Error(e) = result {
            assert_eq!(
                e,
                "error in EventService for Peer LocalError(\"try_event error\")"
            );
        } else {
            assert!(false);
        }
    }
}
