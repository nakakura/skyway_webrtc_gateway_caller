use async_trait::async_trait;
use shaku::*;
use skyway_webrtc_gateway_api::media;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::media::service::MediaApi;
use crate::domain::webrtc::media::value_object::{
    AnswerQuery, AnswerResponse, AnswerResult, CallQuery, MediaConnectionEventEnum,
    MediaConnectionId, MediaConnectionIdWrapper, MediaConnectionStatus, MediaId, RtcpId,
};
use crate::error;

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
    async fn create_media(&self, is_video: bool) -> Result<SocketInfo<MediaId>, error::Error> {
        media::open_media_socket(is_video).await
    }

    async fn delete_media(&self, media_id: &MediaId) -> Result<(), error::Error> {
        media::delete_media(&media_id).await
    }

    async fn create_rtcp(&self) -> Result<SocketInfo<RtcpId>, error::Error> {
        media::open_rtcp_socket().await
    }

    async fn delete_rtcp(&self, rtcp_id: RtcpId) -> Result<RtcpId, error::Error> {
        let _ = media::delete_rtcp(&rtcp_id).await?;
        Ok(rtcp_id)
    }

    async fn call(&self, call_query: CallQuery) -> Result<MediaConnectionIdWrapper, error::Error> {
        Ok(media::call(&call_query).await?.params)
    }

    async fn answer(
        &self,
        media_connection_id: &MediaConnectionId,
        answer_query: AnswerQuery,
    ) -> Result<AnswerResult, error::Error> {
        let result: AnswerResponse = media::answer(&media_connection_id, &answer_query).await?;
        let answer_result = AnswerResult {
            media_connection_id: media_connection_id.clone(),
            send_sockets: Some(result.params.clone()),
            recv_sockets: answer_query.redirect_params,
        };
        Ok(answer_result)
    }

    async fn event(
        &self,
        media_connection_id: &MediaConnectionId,
    ) -> Result<MediaConnectionEventEnum, error::Error> {
        media::event(&media_connection_id).await
    }

    async fn status(
        &self,
        media_connection_id: &MediaConnectionId,
    ) -> Result<MediaConnectionStatus, error::Error> {
        media::status(media_connection_id).await
    }
}
