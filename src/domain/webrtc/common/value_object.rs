// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、特定のAPIに限定されないものを利用する。

/// ID associated with PeerObject
pub use skyway_webrtc_gateway_api::prelude::PeerId;
/// Pair of PeerId and Token
pub use skyway_webrtc_gateway_api::prelude::PeerInfo;
/// Dummy id for sockets which don't have id
pub use skyway_webrtc_gateway_api::prelude::PhantomId;
/// Trait for Serialize Ids
pub use skyway_webrtc_gateway_api::prelude::SerializableId;
/// Trait for Serialize sockets
pub use skyway_webrtc_gateway_api::prelude::SerializableSocket;
/// Socket Information
pub use skyway_webrtc_gateway_api::prelude::SocketInfo;
/// Token issued to prevent misuse of PeerObject
pub use skyway_webrtc_gateway_api::prelude::Token;
