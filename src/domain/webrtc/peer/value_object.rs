// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
// これらは単なるパラメータであり、値自体のvalidationはskyway-webrtc-gateway-api crate内で行われる
// value objectとして利用すべきなのは、PeerId, Token, PeerInfoのみである

/// ID associated with PeerObject
pub use skyway_webrtc_gateway_api::peer::PeerId;
/// Pair of PeerId and Token
pub use skyway_webrtc_gateway_api::peer::PeerInfo;
/// Token to avoid using a PeerObject without ownership
pub use skyway_webrtc_gateway_api::peer::Token;
