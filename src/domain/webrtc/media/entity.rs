// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use serde::{Deserialize, Serialize};

use crate::domain::webrtc::media::value_object::{MediaConnectionId, MediaId, RtcpId};

// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。

/// Parameter for POST /media/connections/{media_connection_id}/answer
pub use skyway_webrtc_gateway_api::media::AnswerQuery;
/// Response from POST /media/connections/{media_connection_id}/answer
pub use skyway_webrtc_gateway_api::media::AnswerResponse;
/// Internal parameter of AnswerResponse
pub use skyway_webrtc_gateway_api::media::AnswerResponseParams;
/// Parameter for POST /media/connections
pub use skyway_webrtc_gateway_api::media::CallQuery;
/// Response from POST /media/connections
pub use skyway_webrtc_gateway_api::media::CallResponse;
/// WebRTC Media Constraints
pub use skyway_webrtc_gateway_api::media::Constraints;
/// Enum represents events of MediaConnection
pub use skyway_webrtc_gateway_api::media::MediaConnectionEventEnum;
/// Wrapper to adapt to JSON format
pub use skyway_webrtc_gateway_api::media::MediaConnectionIdWrapper;
/// Shows status of DataConnection
pub use skyway_webrtc_gateway_api::media::MediaConnectionStatus;
/// Parameters for sending media
pub use skyway_webrtc_gateway_api::media::MediaParams;
/// Shows to which socket media should be redirected.
pub use skyway_webrtc_gateway_api::media::RedirectParameters;
/// Shows ssrc(Synchrozination Source) information
pub use skyway_webrtc_gateway_api::media::SsrcPair;

// JSONとしてserializeする際に{media_id: ...}とフォーマットするためにラッピングする
/// Wrapper to adapt to JSON format
#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug, Clone)]
pub struct MediaIdWrapper {
    pub media_id: MediaId,
}

// JSONとしてserializeする際に{rtcp_id: ...}とフォーマットするためにラッピングする
/// Wrapper to adapt to JSON format
#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug, Clone)]
pub struct RtcpIdWrapper {
    pub rtcp_id: RtcpId,
}

// skyway-webrtc-gateway crateのAnswerで帰ってきたパラメータにはMediaConnectionIdが含まれない。
// エンドユーザはMediaConnectionIdが含まれていたほうが便利であると考えられるので、含めた形で再定義する
/// Result of Answer
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AnswerResult {
    pub media_connection_id: MediaConnectionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_sockets: Option<AnswerResponseParams>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recv_sockets: Option<RedirectParameters>,
}
