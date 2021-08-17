// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

use serde::{Deserialize, Serialize};

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
