use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::usecase::service::EventListener;
use crate::application::usecase::value_object::ResponseMessage;
use crate::di::ApplicationStateContainer;
use crate::domain::data::service::DataApi;
use crate::domain::utility::ApplicationState;
use crate::prelude::{DataConnectionEventEnum, ResponseMessageBodyEnum};

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    api: Arc<dyn DataApi>,
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
            let event = self.api.event(params.clone()).await;
            match event {
                Ok(DataConnectionEventEnum::CLOSE(ref _data_connection_id)) => {
                    let message = ResponseMessage::Success(ResponseMessageBodyEnum::DataEvent(
                        event.unwrap().clone(),
                    ));
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(event) => {
                    let message =
                        ResponseMessage::Success(ResponseMessageBodyEnum::DataEvent(event.clone()));
                    let _ = event_tx.send(message).await;
                }
                Err(e) => {
                    let message = serde_json::to_string(&e).unwrap();
                    return ResponseMessage::Error(message);
                }
            }
        }

        ResponseMessage::Success(ResponseMessageBodyEnum::DataEvent(
            DataConnectionEventEnum::TIMEOUT,
        ))
    }
}

#[cfg(test)]
mod test_data_event {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use skyway_webrtc_gateway_api::error;

    use super::*;
    use crate::di::DataEventServiceContainer;
    use crate::domain::data::service::MockDataApi;
    use crate::prelude::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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
        let mut mock = MockDataApi::default();
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
        let param = serde_json::to_value(DataConnectionIdWrapper {
            data_connection_id: data_connection_id.clone(),
        })
        .unwrap();
        tokio::spawn(async {
            // Mockを埋め込んだEventServiceを生成
            let module = DataEventServiceContainer::builder()
                .with_component_override::<dyn DataApi>(Box::new(mock))
                .build();
            let event_service: &dyn EventListener = module.resolve_ref();

            let _message = event_service.execute(event_tx, param).await;
        });

        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            ResponseMessage::Success(ResponseMessageBodyEnum::DataEvent(
                DataConnectionEventEnum::OPEN(data_connection_id.clone())
            ))
        );

        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            ResponseMessage::Success(ResponseMessageBodyEnum::DataEvent(
                DataConnectionEventEnum::CLOSE(data_connection_id.clone())
            ))
        );
    }

    #[tokio::test]
    async fn recv_error() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let data_connection_id =
            DataConnectionId::try_create("dc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 1回目はOPEN, 2回目はCLOSEイベントを返すMockを作る
        let mut mock = MockDataApi::default();
        mock.expect_event()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // 実行
        let param = serde_json::to_value(DataConnectionIdWrapper {
            data_connection_id: data_connection_id.clone(),
        })
        .unwrap();
        tokio::spawn(async {
            // Mockを埋め込んだEventServiceを生成
            let module = DataEventServiceContainer::builder()
                .with_component_override::<dyn DataApi>(Box::new(mock))
                .build();
            let event_service: &dyn EventListener = module.resolve_ref();

            let message = event_service.execute(event_tx, param).await;
            assert_eq!(
                message,
                ResponseMessage::Error(
                    "{\"reason\":\"InternalError\",\"message\":\"error\"}".into()
                )
            );
        });

        let event = event_rx.recv().await;
        assert_eq!(event, None);
    }
}
