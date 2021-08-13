use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::usecase::service::EventListener;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::media::service::MediaApi;
use crate::domain::media::value_object::MediaConnectionEventEnum;
use crate::domain::utility::ApplicationState;
use crate::prelude::ResponseMessageBodyEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
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
                Ok(MediaConnectionEventEnum::CLOSE(ref _media_connection_id)) => {
                    let message = ResponseMessage::Success(ResponseMessageBodyEnum::Media(
                        MediaResponseMessageBodyEnum::Event(event.unwrap().clone()),
                    ));
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(MediaConnectionEventEnum::TIMEOUT) => {
                    // TIMEOUTはユーザに通知する必要がない
                }
                Ok(event) => {
                    let message = ResponseMessage::Success(ResponseMessageBodyEnum::Media(
                        MediaResponseMessageBodyEnum::Event(event),
                    ));
                    let _ = event_tx.send(message).await;
                }
                Err(e) => {
                    let message = serde_json::to_string(&e).unwrap();
                    let message = ResponseMessage::Error(message);
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
            }
        }

        ResponseMessage::Success(ResponseMessageBodyEnum::Media(
            MediaResponseMessageBodyEnum::Event(MediaConnectionEventEnum::TIMEOUT),
        ))
    }
}

#[cfg(test)]
mod test_delete_media {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::MediaEventServiceContainer;
    use crate::domain::media::service::MockMediaApi;
    use crate::domain::media::value_object::{MediaConnectionId, MediaConnectionIdWrapper};
    use crate::error;
    use crate::infra::utility::ApplicationStateAlwaysFalseImpl;
    use crate::prelude::ResponseMessageBodyEnum;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    // Eventの監視ループを抜けるタイミングは3つあり、3つともテストする
    // CLOSE Eventを受信してループを抜ける場合
    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        let media_connection_id =
            MediaConnectionId::try_create("mc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();
        let ready_event = MediaConnectionEventEnum::READY(media_connection_id.clone());
        let close_event = MediaConnectionEventEnum::CLOSE(media_connection_id.clone());

        // 1回目はREADY, 2回目はCLOSEイベントを返すMockを作る
        let mut counter = 0;
        let mut mock = MockMediaApi::default();
        mock.expect_event().returning(move |_| {
            if counter == 0 {
                counter += 1;
                return Ok(ready_event.clone());
            } else {
                return Ok(close_event.clone());
            }
        });

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // 実行
        let param = serde_json::to_value(MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        })
        .unwrap();

        // Mockを埋め込んだEventServiceを生成
        let module = MediaEventServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // CLOSE Eventで抜けた場合はCLOSE Eventが帰ってくる
        let message = event_service.execute(event_tx, param).await;
        assert_eq!(
            message,
            ResponseMessage::Success(ResponseMessageBodyEnum::Media(
                MediaResponseMessageBodyEnum::Event(MediaConnectionEventEnum::CLOSE(
                    media_connection_id.clone()
                ))
            ))
        );

        // close eventが発火してevent_serviceが終了しているのでこの行に処理が回る
        // 1回目はready eventが帰ってくる
        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            ResponseMessage::Success(ResponseMessageBodyEnum::Media(
                MediaResponseMessageBodyEnum::Event(MediaConnectionEventEnum::READY(
                    media_connection_id.clone()
                ))
            ))
        );

        // 2回目はready eventが帰ってくる
        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            ResponseMessage::Success(ResponseMessageBodyEnum::Media(
                MediaResponseMessageBodyEnum::Event(MediaConnectionEventEnum::CLOSE(
                    media_connection_id.clone()
                ))
            ))
        );
    }

    // Error Eventを受信してループを抜ける場合
    #[tokio::test]
    async fn recv_error() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let media_connection_id =
            MediaConnectionId::try_create("mc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 1回目はOPEN, 2回目はCLOSEイベントを返すMockを作る
        let mut mock = MockMediaApi::default();
        mock.expect_event()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // Mockを埋め込んだEventServiceを生成
        let module = MediaEventServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 実行
        let param = serde_json::to_value(MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        })
        .unwrap();

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // ERRORが発生してループを抜けたErrorが帰ってくる
        let message = event_service.execute(event_tx, param).await;
        assert_eq!(
            message,
            ResponseMessage::Error("{\"reason\":\"InternalError\",\"message\":\"error\"}".into())
        );

        // 発生したERRORを受け取る
        let event = event_rx.recv().await.unwrap();
        assert_eq!(
            event,
            ResponseMessage::Error("{\"reason\":\"InternalError\",\"message\":\"error\"}".into())
        );
    }

    // loopの継続判定がfalseになって抜ける場合
    #[tokio::test]
    async fn loop_exit() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // create params
        let media_connection_id =
            MediaConnectionId::try_create("mc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 1回目はOPEN, 2回目はCLOSEイベントを返すMockを作る
        let mut mock = MockMediaApi::default();
        mock.expect_event()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseMessage>(10);

        // Mockを埋め込んだEventServiceを生成
        let module = MediaEventServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .with_component_override::<dyn ApplicationState>(Box::new(
                ApplicationStateAlwaysFalseImpl {},
            ))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 実行
        let param = serde_json::to_value(MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        })
        .unwrap();

        // event_serviceはループを抜けるときに最後のEVENTを返す
        // Application Stateがfalseを返すことによってループを抜けた場合は、TIMEOUTが帰ってくる
        let message = event_service.execute(event_tx, param).await;
        assert_eq!(
            message,
            ResponseMessage::Success(ResponseMessageBodyEnum::Media(
                MediaResponseMessageBodyEnum::Event(MediaConnectionEventEnum::TIMEOUT)
            ))
        );

        // event発生前にApplicationStateによりloopを抜けている
        let event = event_rx.recv().await;
        assert_eq!(event, None);
    }
}
