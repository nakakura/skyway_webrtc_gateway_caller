// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use skyway_webrtc_gateway_api::prelude::SerializableSocket;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::media::service::MediaApi;
use crate::error;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。
pub use skyway_webrtc_gateway_api::media::{
    AnswerQuery, AnswerResponse, AnswerResponseParams, CallQuery, CallResponse, Constraints,
    MediaConnectionEventEnum, MediaConnectionId, MediaConnectionIdWrapper, MediaConnectionStatus,
    MediaId, MediaParams, RedirectParameters, RtcpId, SsrcPair,
};

/// JSONとしてserializeする際に{media_id: ...}とフォーマットするためにラッピングする
#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug, Clone)]
pub struct MediaIdWrapper {
    pub media_id: MediaId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MediaSocket(pub(crate) SocketInfo<MediaId>);

#[test]
fn media_socket_serialize_deserialize() {
    use crate::domain::webrtc::common::value_object::SerializableSocket;

    let sock = SocketInfo::<MediaId>::try_create(
        Some("vi-4d053831-5dc2-461b-a358-d062d6115216".into()),
        "127.0.0.1",
        5000,
    )
    .unwrap();
    let socket = MediaSocket(sock);
    let message = serde_json::to_string(&socket).unwrap();

    let result: MediaSocket = serde_json::from_str(&message).unwrap();
    assert_eq!(result, socket);
}

//　これらの各メソッドは、application::media内のUnit Testで間接的にテストされている
impl MediaSocket {
    pub async fn try_create(api: Arc<dyn MediaApi>, is_video: bool) -> Result<Self, error::Error> {
        let socket = api.create_media(is_video).await?;
        Ok(MediaSocket(socket))
    }

    pub async fn try_delete(
        api: Arc<dyn MediaApi>,
        media_id: MediaId,
    ) -> Result<MediaId, error::Error> {
        api.delete_media(media_id).await
    }

    pub fn get_id(&self) -> Option<MediaId> {
        self.0.get_id()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RtcpSocket(pub(crate) SocketInfo<RtcpId>);

//　これらの各メソッドは、application::media内のUnit Testで間接的にテストされている
impl RtcpSocket {
    pub async fn try_create(api: Arc<dyn MediaApi>) -> Result<Self, error::Error> {
        let socket = api.create_rtcp().await?;
        Ok(RtcpSocket(socket))
    }

    pub async fn try_delete(
        api: Arc<dyn MediaApi>,
        rtcp_id: RtcpId,
    ) -> Result<RtcpId, error::Error> {
        api.delete_rtcp(rtcp_id).await
    }

    pub fn get_id(&self) -> Option<RtcpId> {
        self.0.get_id()
    }
}

/// JSONとしてserializeする際に{rtcp_id: ...}とフォーマットするためにラッピングする
#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug, Clone)]
pub struct RtcpIdWrapper {
    pub rtcp_id: RtcpId,
}

/// skyway-webrtc-gateway crateのAnswerで帰ってきたパラメータにはMediaConnectionIdが含まれない。
/// エンドユーザはMediaConnectionIdが含まれていたほうが便利であると考えられるので、含めた形で再定義する
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AnswerResult {
    pub media_connection_id: MediaConnectionId,
    pub send_sockets: Option<AnswerResponseParams>,
    pub recv_sockets: Option<RedirectParameters>,
}
