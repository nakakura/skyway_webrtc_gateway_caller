use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::media;

use crate::domain::media::service::MediaApi;
use crate::domain::media::value_object::{CallQuery, MediaConnectionIdWrapper, MediaId, RtcpId};
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
}
