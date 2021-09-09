use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{DataResponseMessageBodyEnum, ResponseMessage};
use crate::application::usecase::service::EventListener;
use crate::domain::state::ApplicationState;
use crate::domain::webrtc::data::entity::{DataConnection, DataConnectionIdWrapper};
use crate::domain::webrtc::data::repository::DataRepository;
use crate::domain::webrtc::data::value_object::DataConnectionId;
use crate::prelude::DataConnectionEventEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    api: Arc<dyn DataRepository>,
    #[shaku(inject)]
    state: Arc<dyn ApplicationState>,
}

impl EventService {
    async fn listen(
        &self,
        event_tx: mpsc::Sender<ResponseMessage>,
        data_connection_id: DataConnectionId,
    ) -> ResponseMessage {
        while self.state.is_running() {
            let event = DataConnection::try_event(self.api.clone(), &data_connection_id).await;
            match event {
                Ok(DataConnectionEventEnum::CLOSE(data_connection_id)) => {
                    let message = DataResponseMessageBodyEnum::Event(
                        DataConnectionEventEnum::CLOSE(data_connection_id),
                    )
                    .create_response_message();
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(DataConnectionEventEnum::TIMEOUT) => {
                    // TIMEOUTはユーザに通知する必要がない
                }
                Ok(event) => {
                    let message =
                        DataResponseMessageBodyEnum::Event(event.clone()).create_response_message();
                    let _ = event_tx.send(message).await;
                }
                Err(e) => {
                    let message = format!("error in EventListener for data. {:?}", e);
                    let message = ResponseMessage::Error(message);
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
            }
        }

        DataResponseMessageBodyEnum::Event(DataConnectionEventEnum::TIMEOUT)
            .create_response_message()
    }
}

#[async_trait]
impl EventListener for EventService {
    async fn execute(
        &self,
        event_tx: mpsc::Sender<ResponseMessage>,
        params: Parameter,
    ) -> ResponseMessage {
        let data_connection_id_wrapper = params.deserialize::<DataConnectionIdWrapper>();
        if data_connection_id_wrapper.is_err() {
            let message = format!(
                "invalid data_connection_id {:?}",
                data_connection_id_wrapper.err()
            );
            return ResponseMessage::Error(message);
        }

        let data_connection_id = data_connection_id_wrapper.unwrap().data_connection_id;
        self.listen(event_tx, data_connection_id).await
    }
}

#[cfg(test)]
mod test_data_event {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::di::DataEventServiceContainer;
    use crate::domain::webrtc::data::repository::MockDataRepository;
    use crate::error;
    use crate::infra::state::ApplicationStateAlwaysFalseImpl;
    use crate::prelude::*;

    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    // Eventの監視ループを抜けるタイミングは3つあり、3つともテストする
    // CLOSE Eventを受信してループを抜ける場合
    #[tokio::test]
    async fn connect_and_close() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();
        let open_event = DataConnectionEventEnum::OPEN(data_connection_id.clone());
        let close_event = DataConnectionEventEnum::CLOSE(data_connection_id.clone());

        // 1回目はOPEN, 2回目はCLOSEイベントを返すMockを作る
        let mut counter = 0;
        let mut mock = MockDataRepository::default();
        mock.expect_event().returning(move |_| {
            if counter == 0 {
                counter += 1;
                return Ok(open_event.clone());
            } else {
                return Ok(close_event.clone());
            }
        });

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // 実行
        let param = Parameter(
            serde_json::to_value(DataConnectionIdWrapper {
                data_connection_id: data_connection_id.clone(),
            })
            .unwrap(),
        );

        // Mockを埋め込んだEventServiceを生成
        let module = DataEventServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // CLOSE Eventで抜けた場合はCLOSE Eventが帰ってくる
        let message = event_service.execute(event_tx, param).await;
        assert_eq!(
            message,
            DataResponseMessageBodyEnum::Event(DataConnectionEventEnum::CLOSE(
                data_connection_id.clone()
            ))
            .create_response_message()
        );

        // event_service内から送信されたevent
        // 1回目はOPEN Eventが送信されている
        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            DataResponseMessageBodyEnum::Event(DataConnectionEventEnum::OPEN(
                data_connection_id.clone()
            ))
            .create_response_message()
        );

        // event_service内から送信されたevent
        // 2回目はCLOSE Eventが送信されている
        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            DataResponseMessageBodyEnum::Event(DataConnectionEventEnum::CLOSE(data_connection_id))
                .create_response_message()
        );
    }

    // Error Eventを受信してループを抜ける場合
    #[tokio::test]
    async fn recv_error() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 1回目はOPEN, 2回目はCLOSEイベントを返すMockを作る
        let mut mock = MockDataRepository::default();
        mock.expect_event()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // Mockを埋め込んだEventServiceを生成
        let module = DataEventServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 実行
        let param = Parameter(
            serde_json::to_value(DataConnectionIdWrapper {
                data_connection_id: data_connection_id.clone(),
            })
            .unwrap(),
        );

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // ERRORが発生してループを抜けたErrorが帰ってくる
        let message = event_service.execute(event_tx, param).await;
        assert_eq!(
            message,
            ResponseMessage::Error("error in EventListener for data. LocalError(\"error\")".into())
        );

        // 発生したERRORを受け取る
        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            ResponseMessage::Error("error in EventListener for data. LocalError(\"error\")".into())
        );
    }

    // loopの継続判定がfalseになって抜ける場合
    #[tokio::test]
    async fn loop_exit() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 1回目はOPEN, 2回目はCLOSEイベントを返すMockを作る
        let mut mock = MockDataRepository::default();
        mock.expect_event()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // Mockを埋め込んだEventServiceを生成
        let module = DataEventServiceContainer::builder()
            .with_component_override::<dyn DataRepository>(Box::new(mock))
            .with_component_override::<dyn ApplicationState>(Box::new(
                ApplicationStateAlwaysFalseImpl {},
            ))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 実行
        let param = serde_json::to_value(DataConnectionIdWrapper {
            data_connection_id: data_connection_id.clone(),
        })
        .unwrap();

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // Application Stateがfalseを返すことによってループを抜けた場合は、TIMEOUTが帰ってくる
        let message = event_service.execute(event_tx, Parameter(param)).await;
        assert_eq!(
            message,
            DataResponseMessageBodyEnum::Event(DataConnectionEventEnum::TIMEOUT)
                .create_response_message()
        );

        // event発生前にApplicationStateによりloopを抜けている
        let event = event_rx.recv().await;
        assert_eq!(event, None);
    }
}
