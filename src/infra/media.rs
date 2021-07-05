use async_trait::async_trait;
use serde_json::Value;
use shaku::*;
use skyway_webrtc_gateway_api::media;

use crate::domain::media::service::MediaApi;
use crate::domain::media::value_object::MediaId;
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
}
