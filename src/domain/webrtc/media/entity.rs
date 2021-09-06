// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use std::net::IpAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use skyway_webrtc_gateway_api::prelude::SerializableSocket;

use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::media::repository::MediaRepository;
use crate::domain::webrtc::media::value_object::{MediaConnectionId, MediaId, RtcpId};
use crate::error;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。
pub use skyway_webrtc_gateway_api::media::{
    AnswerQuery, AnswerResponse, AnswerResponseParams, CallQuery, CallResponse, Constraints,
    MediaConnectionEventEnum, MediaConnectionIdWrapper, MediaConnectionStatus, MediaParams,
    RedirectParameters, SsrcPair,
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
    pub async fn try_create(
        api: Arc<dyn MediaRepository>,
        is_video: bool,
    ) -> Result<Self, error::Error> {
        let socket = api.create_media(is_video).await?;
        Ok(MediaSocket(socket))
    }

    pub async fn try_delete(
        api: Arc<dyn MediaRepository>,
        media_id: &MediaId,
    ) -> Result<(), error::Error> {
        api.delete_media(media_id).await
    }

    pub fn get_id(&self) -> Option<MediaId> {
        self.0.get_id()
    }

    pub fn ip(&self) -> IpAddr {
        self.0.ip()
    }

    pub fn port(&self) -> u16 {
        self.0.port()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RtcpSocket(pub(crate) SocketInfo<RtcpId>);

//　これらの各メソッドは、application::media内のUnit Testで間接的にテストされている
impl RtcpSocket {
    pub async fn try_create(api: Arc<dyn MediaRepository>) -> Result<Self, error::Error> {
        let socket = api.create_rtcp().await?;
        Ok(RtcpSocket(socket))
    }

    pub async fn try_delete(
        api: Arc<dyn MediaRepository>,
        rtcp_id: &RtcpId,
    ) -> Result<(), error::Error> {
        api.delete_rtcp(rtcp_id).await
    }

    pub fn get_id(&self) -> Option<RtcpId> {
        self.0.get_id()
    }

    pub fn ip(&self) -> IpAddr {
        self.0.ip()
    }

    pub fn port(&self) -> u16 {
        self.0.port()
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

pub struct MediaConnection {
    api: Arc<dyn MediaRepository>,
    media_connection_id: MediaConnectionId,
}

impl MediaConnection {
    pub async fn try_create(
        api: Arc<dyn MediaRepository>,
        query: CallQuery,
    ) -> Result<CallResponse, error::Error> {
        api.call(query).await
    }

    pub async fn find(
        api: Arc<dyn MediaRepository>,
        media_connection_id: MediaConnectionId,
    ) -> Result<(Self, MediaConnectionStatus), error::Error> {
        let status = api.status(&media_connection_id).await?;
        Ok((
            Self {
                api,
                media_connection_id,
            },
            status,
        ))
    }

    pub async fn try_answer(&self, query: &AnswerQuery) -> Result<AnswerResponse, error::Error> {
        self.api.answer(&self.media_connection_id, query).await
    }

    pub async fn try_event(
        api: Arc<dyn MediaRepository>,
        media_connection_id: &MediaConnectionId,
    ) -> Result<MediaConnectionEventEnum, error::Error> {
        api.event(media_connection_id).await
    }

    pub fn media_connection_id(&self) -> &MediaConnectionId {
        &self.media_connection_id
    }
}