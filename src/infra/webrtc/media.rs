use async_trait::async_trait;
use shaku::*;
use skyway_webrtc_gateway_api::media;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::domain::webrtc::media::value_object::{
    AnswerQuery, AnswerResponse, CallQuery, CallResponse, MediaConnectionEventEnum,
    MediaConnectionId, MediaConnectionStatus, MediaId, RtcpId,
};
use crate::error;

// skyway_webrtc_gateway_apiの関数の単純なラッパ
#[derive(Component)]
#[shaku(interface = MediaRepository)]
pub(crate) struct MediaRepositoryImpl;

impl Default for MediaRepositoryImpl {
    fn default() -> Self {
        MediaRepositoryImpl {}
    }
}

// FIXME: シンプルなので単体テストはしていない。結合試験のみ
#[async_trait]
impl MediaRepository for MediaRepositoryImpl {
    async fn create_media(&self, is_video: bool) -> Result<SocketInfo<MediaId>, error::Error> {
        media::open_media_socket(is_video).await
    }

    async fn delete_media(&self, media_id: &MediaId) -> Result<(), error::Error> {
        media::delete_media(&media_id).await
    }

    async fn create_rtcp(&self) -> Result<SocketInfo<RtcpId>, error::Error> {
        media::open_rtcp_socket().await
    }

    async fn delete_rtcp(&self, rtcp_id: &RtcpId) -> Result<(), error::Error> {
        media::delete_rtcp(rtcp_id).await
    }

    async fn call(&self, call_query: CallQuery) -> Result<CallResponse, error::Error> {
        media::call(&call_query).await
    }

    async fn answer(
        &self,
        media_connection_id: &MediaConnectionId,
        answer_query: &AnswerQuery,
    ) -> Result<AnswerResponse, error::Error> {
        media::answer(media_connection_id, answer_query).await
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
