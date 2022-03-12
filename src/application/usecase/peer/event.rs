use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{PeerResponse, ResponseResult};
use crate::application::usecase::service::EventListener;
use crate::domain::state::ApplicationState;
use crate::domain::webrtc::peer::entity::PeerEventEnum;
use crate::domain::webrtc::peer::repository::PeerRepository;
use crate::domain::webrtc::peer::value_object::PeerInfo;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    repository: Arc<dyn PeerRepository>,
    #[shaku(inject)]
    state: Arc<dyn ApplicationState>,
}

#[async_trait]
impl EventListener for EventService {
    async fn execute(
        &self,
        event_tx: mpsc::Sender<ResponseResult>,
        params: Parameter,
    ) -> ResponseResult {
        // 汎用的なDTOオブジェクトであるParameterから必要な値を取り出せるかチェックするのはアプリケーション層の責務である
        let peer_info = params.deserialize::<PeerInfo>();
        // パースエラーの場合はエラーを示すenumを返す
        if peer_info.is_err() {
            let message = format!("invalid peer_info {:?}", peer_info.err().unwrap());
            let message = ResponseResult::Error(message);
            // イベントとして通知する
            let _ = event_tx.send(message.clone()).await;
            // 直接的な実行結果としても返しておく
            return message;
        }

        let peer_info = peer_info.unwrap();

        while self.state.is_running() {
            let event = self.repository.event(peer_info.clone()).await;
            match event {
                Ok(PeerEventEnum::CLOSE(event)) => {
                    let message = PeerResponse::Event(PeerEventEnum::CLOSE(event).clone())
                        .create_response_message();
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(PeerEventEnum::TIMEOUT) => {
                    // TIMEOUTはユーザに通知する必要がない
                }
                Ok(event) => {
                    let message = PeerResponse::Event(event).create_response_message();
                    let _ = event_tx.send(message.clone()).await;
                }
                Err(e) => {
                    let message = format!("error in EventService for Peer {:?}", e);
                    let message = ResponseResult::Error(message);
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
            }
        }
        PeerResponse::Event(PeerEventEnum::TIMEOUT).create_response_message()
    }
}

#[cfg(test)]
mod test_peer_event {
    use std::sync::Mutex;

    use crate::di::PeerEventServiceContainer;
    use crate::domain::webrtc::data::entity::DataConnectionIdWrapper;
    use crate::domain::webrtc::data::value_object::DataConnectionId;
    use crate::domain::webrtc::peer::entity::{PeerCloseEvent, PeerConnectionEvent};
    use crate::domain::webrtc::peer::repository::MockPeerRepository;
    use crate::error;
    use crate::infra::state::ApplicationStateAlwaysFalseImpl;

    use super::*;

    // 成功する場合
    #[tokio::test]
    async fn success() {
        // いくつかのイベントを取得した後、CLOSEが発火すると監視終了
        // このテストでは、CONNECT, TIMEOUT, CLOSEの順に受信するものとする

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // イベントの作成
        // CONNECTイベント
        let connect_event = PeerEventEnum::CONNECTION(PeerConnectionEvent {
            params: peer_info.clone(),
            data_params: DataConnectionIdWrapper {
                data_connection_id: DataConnectionId::try_create(
                    "dc-102127d9-30de-413b-93f7-41a33e39d82b",
                )
                .unwrap(),
            },
        });
        // CLOSEイベント
        let close_event = PeerEventEnum::CLOSE(PeerCloseEvent {
            params: peer_info.clone(),
        });

        // 期待値を生成しておく
        // CONNECTのイベントの後受け取るであろう値
        let expected_connect = PeerResponse::Event(connect_event.clone()).create_response_message();
        // CLOSEのイベントの後受け取るであろう値
        let expected_close = PeerResponse::Event(close_event.clone()).create_response_message();

        // イベントを3つ返すmockを作成
        // 3つの処理を分けるためのカウンタ
        let counter = Mutex::new(0u8);
        let mut mock = MockPeerRepository::default();
        mock.expect_event().returning(move |_| {
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

        // event_serviceを生成
        let module = &PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 実行のためのパラメータ生成
        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseResult>(10);
        // execute
        let result = event_service
            .execute(
                event_tx,
                Parameter(serde_json::to_value(&peer_info).unwrap()),
            )
            .await;

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

    // event apiはcloseが発火するか、stateがfalseを返すまで繰り返しアクセスされる
    // 最初からstateがfalseを返すのでイベントを取得しに行かないパターンのテスト
    #[tokio::test]
    async fn exit_due_to_state() {
        // 呼ばれないことを確認するため、呼ばれたらクラッシュするモックを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_event().returning(move |_| {
            assert!(false);
            unreachable!()
        });

        // EventServiceを生成
        // stateは必ずfalseを返すモックを挿入
        let module = &PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .with_component_override::<dyn ApplicationState>(Box::new(
                ApplicationStateAlwaysFalseImpl {},
            ))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (event_tx, _) = mpsc::channel::<ResponseResult>(10);

        // execute
        let result = event_service
            .execute(
                event_tx,
                Parameter(serde_json::to_value(&peer_info).unwrap()),
            )
            .await;

        // stateによりイベントループを抜けた場合、最後はTIMEOUTを返す
        assert_eq!(
            result,
            PeerResponse::Event(PeerEventEnum::TIMEOUT).create_response_message()
        );
    }

    // エンドユーザから与えられたjsonが間違っており、イベントを取得できない場合
    #[tokio::test]
    async fn invalid_json() {
        // 呼ばれないことを確認するため、呼ばれたらクラッシュするモックを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_event().returning(move |_| {
            assert!(false);
            unreachable!()
        });

        // event_serviceを生成
        let module = &PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 実行時の引数を生成
        // event_serviceの引数は、JSON化されたPeerInfoとevent senderであるが、なぜかbool値が入ってきたケース
        let param = Parameter(serde_json::Value::Bool(true));
        let (event_tx, _) = mpsc::channel::<ResponseResult>(10);

        // execute
        let result = event_service.execute(event_tx, param).await;

        if let ResponseResult::Error(message) = result {
            assert_eq!(
                &message,
                "invalid peer_info SerdeError { error: Error(\"invalid type: boolean `true`, expected struct PeerInfo\", line: 0, column: 0) }"
            );
        } else {
            assert!(false);
        }
    }

    // APIがエラーを帰す場合
    #[tokio::test]
    async fn fail() {
        // errorを返すmockを作成
        let mut mock = MockPeerRepository::default();
        mock.expect_event().returning(move |_| {
            return Err(error::Error::create_local_error("event error"));
        });

        // event_serviceを生成
        let module = &PeerEventServiceContainer::builder()
            .with_component_override::<dyn PeerRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // event_serviceの引数は、JSON化されたPeerInfoとevent senderである
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (event_tx, _) = mpsc::channel::<ResponseResult>(10);

        // execute
        let result = event_service
            .execute(
                event_tx,
                Parameter(serde_json::to_value(&peer_info).unwrap()),
            )
            .await;

        // errorが帰ってくる
        if let ResponseResult::Error(e) = result {
            assert_eq!(
                e,
                "error in EventService for Peer LocalError(\"event error\")"
            );
        } else {
            assert!(false);
        }
    }
}
