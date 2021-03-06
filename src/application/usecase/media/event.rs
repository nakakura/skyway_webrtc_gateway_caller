use std::sync::Arc;

use async_trait::async_trait;
use shaku::*;
use tokio::sync::mpsc;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::{MediaResponse, ResponseResult};
use crate::application::usecase::service::EventListener;
use crate::domain::state::ApplicationState;
use crate::domain::webrtc::media::entity::{MediaConnectionEventEnum, MediaConnectionIdWrapper};
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::domain::webrtc::media::value_object::MediaConnectionId;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = EventListener)]
pub(crate) struct EventService {
    #[shaku(inject)]
    repository: Arc<dyn MediaRepository>,
    #[shaku(inject)]
    state: Arc<dyn ApplicationState>,
}

impl EventService {
    async fn listen(
        &self,
        event_tx: mpsc::Sender<ResponseResult>,
        media_connection_id: MediaConnectionId,
    ) -> ResponseResult {
        while self.state.is_running() {
            let event = self.repository.event(&media_connection_id).await;
            match event {
                Ok(MediaConnectionEventEnum::CLOSE(media_connection_id)) => {
                    let message =
                        MediaResponse::Event(MediaConnectionEventEnum::CLOSE(media_connection_id))
                            .create_response_message();
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
                Ok(MediaConnectionEventEnum::TIMEOUT) => {
                    // TIMEOUTはユーザに通知する必要がない
                }
                Ok(event) => {
                    let message = MediaResponse::Event(event).create_response_message();
                    let _ = event_tx.send(message).await;
                }
                Err(e) => {
                    let message = format!("error in EventListener for Media {:?}", e);
                    let message = ResponseResult::Error(message);
                    let _ = event_tx.send(message.clone()).await;
                    return message;
                }
            }
        }

        MediaResponse::Event(MediaConnectionEventEnum::TIMEOUT).create_response_message()
    }
}

#[async_trait]
impl EventListener for EventService {
    async fn execute(
        &self,
        event_tx: mpsc::Sender<ResponseResult>,
        params: Parameter,
    ) -> ResponseResult {
        let media_connection_id_wrapper = params.deserialize::<MediaConnectionIdWrapper>();
        if media_connection_id_wrapper.is_err() {
            let message = format!(
                "invalid media_connection_id {:?}",
                media_connection_id_wrapper.err()
            );
            return ResponseResult::Error(message);
        }
        let media_connection_id = media_connection_id_wrapper.unwrap().media_connection_id;
        self.listen(event_tx, media_connection_id).await
    }
}

#[cfg(test)]
mod test_delete_media {
    use crate::di::MediaEventServiceContainer;
    use crate::domain::webrtc::media::entity::MediaConnectionStatus;
    use crate::domain::webrtc::media::repository::MockMediaRepository;
    use crate::domain::webrtc::peer::value_object::PeerId;
    use crate::infra::state::ApplicationStateAlwaysFalseImpl;

    use super::*;

    // eventはcloseが発火するか、stateがfalseを返すまで繰り返される
    // このケースはcloseが発火するまでイベントの取得を続ける
    // READY, TIMEOUT, CLOSEの順で発火させる
    #[tokio::test]
    async fn success() {
        let media_connection_id =
            MediaConnectionId::try_create("mc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // 発火させるイベントの準備
        let ready_event = MediaConnectionEventEnum::READY(MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        });
        let close_event = MediaConnectionEventEnum::CLOSE(MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        });

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        let counter_mutex = std::sync::Mutex::new(0u8);
        mock.expect_event().returning(move |_| {
            let mut counter = counter_mutex.lock().unwrap();
            *counter += 1;
            match *counter {
                1 => {
                    // 1回目はREADYを返す
                    return Ok(ready_event.clone());
                }
                2 => {
                    // 2回目はTIMEOUTを返す
                    return Ok(MediaConnectionEventEnum::TIMEOUT);
                }
                3 => {
                    return Ok(close_event.clone());
                    // 3回目はCLOSEを返す
                }
                _ => {
                    // 4回目以降は発火しない
                    unreachable!()
                }
            }
        });
        mock.expect_status().returning(move |_| {
            // MediaConnectionがまだ開いていないというステータスを返す
            return Ok(MediaConnectionStatus {
                metadata: "metadata".to_string(),
                open: true,
                remote_id: PeerId::new("peer_id"),
                ssrc: None,
            });
        });

        let module = &MediaEventServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 引数の生成
        let param = MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        };
        let param = serde_json::to_value(param).unwrap();
        // eventを受け取るためのチャンネルを作成
        let (event_tx, mut event_rx) = mpsc::channel::<ResponseResult>(10);

        //実行
        let result = event_service.execute(event_tx, Parameter(param)).await;

        // CLOSE Eventが発火した場合は最後にCLOSE EVENTが帰る
        assert_eq!(
            result,
            MediaResponse::Event(MediaConnectionEventEnum::CLOSE(MediaConnectionIdWrapper {
                media_connection_id: media_connection_id.clone()
            }))
            .create_response_message()
        );

        // eventが通知されていることを確認
        // 1つめはREADYイベント
        let result = event_rx.recv().await;
        if let Some(result_close_event) = result {
            assert_eq!(
                result_close_event,
                MediaResponse::Event(MediaConnectionEventEnum::READY(MediaConnectionIdWrapper {
                    media_connection_id: media_connection_id.clone()
                }))
                .create_response_message()
            );
        } else {
            assert!(false);
        }

        // 2つめはCLOSEイベント
        let result = event_rx.recv().await;
        if let Some(result_close_event) = result {
            assert_eq!(
                result_close_event,
                MediaResponse::Event(MediaConnectionEventEnum::CLOSE(MediaConnectionIdWrapper {
                    media_connection_id: media_connection_id.clone()
                }))
                .create_response_message()
            );
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
    async fn loop_exit() {
        let media_connection_id =
            MediaConnectionId::try_create("mc-4995f372-fb6a-4196-b30a-ce11e5c7f56c").unwrap();

        // eventを受け取るためのチャンネルを作成
        let (event_tx, _) = mpsc::channel::<ResponseResult>(10);

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_event().returning(move |_| unreachable!());
        mock.expect_status().returning(move |_| {
            // MediaConnectionがまだ開いていないというステータスを返す
            return Ok(MediaConnectionStatus {
                metadata: "metadata".to_string(),
                open: true,
                remote_id: PeerId::new("peer_id"),
                ssrc: None,
            });
        });

        let module = &MediaEventServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            // 常にfalseを返すStateObject
            .with_component_override::<dyn ApplicationState>(Box::new(
                ApplicationStateAlwaysFalseImpl {},
            ))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();

        // 引数の生成
        let param = MediaConnectionIdWrapper {
            media_connection_id: media_connection_id.clone(),
        };
        let param = Parameter(serde_json::to_value(param).unwrap());

        //実行
        let result = event_service.execute(event_tx, param).await;

        // loop exitの場合は最後にTIMEOUTが帰る
        assert_eq!(
            result,
            MediaResponse::Event(MediaConnectionEventEnum::TIMEOUT).create_response_message()
        );
    }

    #[tokio::test]
    async fn invalid_param() {
        // eventを受け取るためのチャンネルを作成
        let (event_tx, _) = mpsc::channel::<ResponseResult>(10);

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_event().returning(move |_| unreachable!());

        let module = &MediaEventServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let event_service: &dyn EventListener = module.resolve_ref();
        let result = event_service
            .execute(event_tx, Parameter(serde_json::Value::Bool(true)))
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let ResponseResult::Error(message) = result {
            assert_eq!(&message, "invalid media_connection_id Some(SerdeError { error: Error(\"invalid type: boolean `true`, expected struct MediaConnectionIdWrapper\", line: 0, column: 0) })");
        } else {
            assert!(false);
        }
    }
}
