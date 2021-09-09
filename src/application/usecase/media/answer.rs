use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use shaku::*;

use crate::application::dto::Parameter;
use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::{MediaResponseMessageBodyEnum, ResponseMessage};
use crate::domain::webrtc::media::entity::{
    AnswerQuery, AnswerResponseParams, AnswerResult, MediaConnection,
};
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::domain::webrtc::media::value_object::MediaConnectionId;
use crate::error;

#[derive(Debug, Serialize, Deserialize)]
struct AnswerParameters {
    media_connection_id: MediaConnectionId,
    answer_query: AnswerQuery,
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct AnswerService {
    #[shaku(inject)]
    api: Arc<dyn MediaRepository>,
}

#[async_trait]
impl Service for AnswerService {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error> {
        let answer_parameters = params.deserialize::<AnswerParameters>()?;
        let (media_connection, status) = MediaConnection::find(
            self.api.clone(),
            answer_parameters.media_connection_id.clone(),
        )
        .await?;
        if !status.open {
            // MediaConnectionが確立前の場合のみanswerメソッドを実行する
            let result = media_connection
                .try_answer(&answer_parameters.answer_query)
                .await?;
            let video_params = result.params.video_id;
            let audio_params = result.params.audio_id;
            let send_socket = if video_params.is_none() && audio_params.is_none() {
                None
            } else {
                Some(AnswerResponseParams {
                    video_id: video_params,
                    audio_id: audio_params,
                })
            };
            let result = AnswerResult {
                media_connection_id: media_connection.media_connection_id().clone(),
                send_sockets: send_socket,
                recv_sockets: answer_parameters.answer_query.redirect_params,
            };
            Ok(MediaResponseMessageBodyEnum::Answer(result).create_response_message())
        } else {
            // 確率後の場合はanswerは行わない
            let message = format!(
                "MediaConnection {} has been already opened.",
                media_connection.media_connection_id().as_str()
            );
            Ok(ResponseMessage::Error(message))
        }
    }
}

#[cfg(test)]
mod test_answer {
    use super::*;
    use crate::di::MediaAnswerServiceContainer;
    use crate::domain::webrtc::media::entity::{
        AnswerResponse, AnswerResponseParams, AnswerResult, Constraints, MediaConnectionStatus,
    };
    use crate::domain::webrtc::media::repository::MockMediaRepository;
    use crate::domain::webrtc::peer::value_object::PeerId;
    use crate::error;
    use crate::prelude::MediaConnectionId;

    #[tokio::test]
    async fn success() {
        // 期待値を生成
        let media_connection_id =
            MediaConnectionId::try_create("mc-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let params = AnswerResult {
            media_connection_id: media_connection_id.clone(),
            send_sockets: None,
            recv_sockets: None,
        };
        let expected =
            MediaResponseMessageBodyEnum::Answer(params.clone()).create_response_message();

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        mock.expect_answer().returning(move |_, _query| {
            let response = AnswerResponse {
                command_type: "ANSWER".to_string(),
                params: AnswerResponseParams {
                    video_id: None,
                    audio_id: None,
                },
            };
            return Ok(response);
        });
        // MediaConnectionの生成にstatusも必要
        let expected_status = MediaConnectionStatus {
            metadata: "metadata".to_string(),
            open: false,
            remote_id: PeerId::new("peer_id"),
            ssrc: None,
        };
        mock.expect_status().returning(move |_| {
            return Ok(expected_status.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaAnswerServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let answer_service: Arc<dyn Service> = module.resolve();

        // 実行パラメータの生成
        let query = AnswerQuery {
            constraints: Constraints {
                video: false,
                videoReceiveEnabled: None,
                audio: false,
                audioReceiveEnabled: None,
                video_params: None,
                audio_params: None,
                metadata: None,
            },
            redirect_params: None,
        };
        let params = AnswerParameters {
            media_connection_id,
            answer_query: query,
        };
        // 実行
        let result = answer_service
            .execute(Parameter(serde_json::to_value(params).unwrap()))
            .await
            .unwrap();

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn already_connected() {
        // 期待値を生成
        let media_connection_id =
            MediaConnectionId::try_create("mc-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let expected = format!(
            "MediaConnection {} has been already opened.",
            media_connection_id.as_str()
        );

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaRepository::default();
        // answerは呼ばれないのでmockingは必要ない
        // 既にopen済みでanswerが必要ないケース
        let expected_status = MediaConnectionStatus {
            metadata: "metadata".to_string(),
            open: true,
            remote_id: PeerId::new("peer_id"),
            ssrc: None,
        };
        mock.expect_status().returning(move |_| {
            return Ok(expected_status.clone());
        });

        // Mockを埋め込んだEventServiceを生成
        let module = MediaAnswerServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let answer_service: Arc<dyn Service> = module.resolve();

        // 実行パラメータの生成
        let query = AnswerQuery {
            constraints: Constraints {
                video: false,
                videoReceiveEnabled: None,
                audio: false,
                audioReceiveEnabled: None,
                video_params: None,
                audio_params: None,
                metadata: None,
            },
            redirect_params: None,
        };
        let params = AnswerParameters {
            media_connection_id,
            answer_query: query,
        };
        // 実行
        let result = answer_service
            .execute(Parameter(serde_json::to_value(params).unwrap()))
            .await
            .unwrap();

        // evaluate
        if let ResponseMessage::Error(message) = result {
            assert_eq!(message, expected);
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn invalid_param() {
        // socketの生成に成功する場合のMockを作成
        // メソッドは呼ばれないので初期化はしないでOK
        let mock = MockMediaRepository::default();

        // Mockを埋め込んだEventServiceを生成
        let module = MediaAnswerServiceContainer::builder()
            .with_component_override::<dyn MediaRepository>(Box::new(mock))
            .build();
        let answer_service: Arc<dyn Service> = module.resolve();

        // 間違ったパラメータで実行
        let result = answer_service
            .execute(Parameter(serde_json::value::Value::Bool(true)))
            .await;

        // 求められるJSONとは異なるのでSerdeErrorが帰る
        if let Err(error::Error::SerdeError { error: _ }) = result {
            assert!(true);
        } else {
            assert!(false);
        }
    }
}
