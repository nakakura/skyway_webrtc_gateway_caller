use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::media;

use crate::domain::webrtc::media::service::MediaApi;
use crate::domain::webrtc::media::value_object::{
    AnswerQuery, AnswerResponse, AnswerResult, CallQuery, MediaConnectionEventEnum,
    MediaConnectionId, MediaConnectionIdWrapper, MediaId, RtcpId,
};
use crate::error;
use crate::prelude::SocketInfo;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = MediaApi)]
pub(crate) struct MediaApiImpl;

impl Default for MediaApiImpl {
    fn default() -> Self {
        MediaApiImpl {}
    }
}

// FIXME: シンプルなので単体テストはしていない。結合試験のみ
#[async_trait]
impl MediaApi for MediaApiImpl {
    async fn create_media(&self, is_video: Value) -> Result<SocketInfo<MediaId>, error::Error> {
        let is_video =
            serde_json::from_value(is_video).map_err(|e| error::Error::SerdeError { error: e })?;
        media::open_media_socket(is_video).await
    }

    async fn delete_media(&self, media_id: Value) -> Result<MediaId, error::Error> {
        let media_id =
            serde_json::from_value(media_id).map_err(|e| error::Error::SerdeError { error: e })?;
        let _ = media::delete_media(&media_id).await?;
        Ok(media_id)
    }

    async fn create_rtcp(&self) -> Result<SocketInfo<RtcpId>, error::Error> {
        media::open_rtcp_socket().await
    }

    async fn delete_rtcp(&self, rtcp_id: Value) -> Result<RtcpId, error::Error> {
        let rtcp_id = serde_json::from_value::<RtcpId>(rtcp_id)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        let _ = media::delete_rtcp(&rtcp_id).await?;
        Ok(rtcp_id)
    }

    async fn call(&self, call_query: Value) -> Result<MediaConnectionIdWrapper, error::Error> {
        let call_query = serde_json::from_value::<CallQuery>(call_query)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        let result = media::call(&call_query).await?;
        Ok(result.params)
    }

    async fn answer(&self, answer_query: Value) -> Result<AnswerResult, error::Error> {
        use serde::Deserialize;
        #[derive(Debug, Deserialize)]
        struct AnswerParameters {
            media_connection_id: MediaConnectionId,
            answer_query: AnswerQuery,
        }
        let answer_parameters = serde_json::from_value::<AnswerParameters>(answer_query)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        let result: AnswerResponse = media::answer(
            &answer_parameters.media_connection_id,
            &answer_parameters.answer_query,
        )
        .await?;

        let answer_result = AnswerResult {
            media_connection_id: answer_parameters.media_connection_id,
            send_sockets: Some(result.params.clone()),
            recv_sockets: answer_parameters.answer_query.redirect_params,
        };
        Ok(answer_result)
    }

    async fn event(
        &self,
        media_connection_id: Value,
    ) -> Result<MediaConnectionEventEnum, error::Error> {
        let media_connection_id = serde_json::from_value::<MediaConnectionId>(media_connection_id)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        media::event(&media_connection_id).await
    }
}
