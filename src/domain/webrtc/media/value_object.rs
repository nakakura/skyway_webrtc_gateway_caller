// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。
// value objectとして利用すべきものは、MediaId, RtcpId, MediaConnectionIdのみである

/// ID associated with MediaConnection
pub use skyway_webrtc_gateway_api::media::MediaConnectionId;
/// ID associated with Media
pub use skyway_webrtc_gateway_api::media::MediaId;
/// ID associated with Rtcp
pub use skyway_webrtc_gateway_api::media::RtcpId;
