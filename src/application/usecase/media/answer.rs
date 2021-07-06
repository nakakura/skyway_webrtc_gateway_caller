use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::*;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::ResponseMessage;
use crate::domain::media::service::MediaApi;
use crate::error;
use crate::prelude::ResponseMessageBodyEnum;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = Service)]
pub(crate) struct AnswerService {
    #[shaku(inject)]
    api: Arc<dyn MediaApi>,
}

#[async_trait]
impl Service for AnswerService {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error> {
        let param = self.api.answer(params).await?;
        Ok(ResponseMessage::Success(
            ResponseMessageBodyEnum::MediaAnswer(param),
        ))
    }
}

#[cfg(test)]
mod test_answer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;
    use crate::di::MediaAnswerServiceContainer;
    use crate::domain::common::value_object::SerializableId;
    use crate::domain::media::service::MockMediaApi;
    use crate::domain::media::value_object::{AnswerResponseParams, MediaId};
    use crate::error;
    use crate::prelude::ResponseMessageBodyEnum;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let video_id = MediaId::try_create("vi-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let audio_id = MediaId::try_create("au-50a32bab-b3d9-4913-8e20-f79c90a6a211").unwrap();
        let params = AnswerResponseParams {
            video_id: Some(video_id.clone()),
            audio_id: Some(audio_id.clone()),
        };
        let expected =
            ResponseMessage::Success(ResponseMessageBodyEnum::MediaAnswer(params.clone()));

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_answer()
            .returning(move |_| return Ok(params.clone()));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaAnswerServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result = crate::application::usecase::service::execute_service(
            create_service,
            serde_json::Value::Bool(true),
        )
        .await;

        // evaluate
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 期待値を生成
        let expected = serde_json::to_string(&error::Error::create_local_error("error")).unwrap();
        let expected = ResponseMessage::Error(expected);

        // socketの生成に成功する場合のMockを作成
        let mut mock = MockMediaApi::default();
        mock.expect_answer()
            .returning(move |_| Err(error::Error::create_local_error("error")));

        // Mockを埋め込んだEventServiceを生成
        let module = MediaAnswerServiceContainer::builder()
            .with_component_override::<dyn MediaApi>(Box::new(mock))
            .build();
        let create_service: &dyn Service = module.resolve_ref();

        // execute
        let result = crate::application::usecase::service::execute_service(
            create_service,
            serde_json::Value::Bool(true),
        )
        .await;

        // evaluate
        assert_eq!(result, expected);
    }
}
