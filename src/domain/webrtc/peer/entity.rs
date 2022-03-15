// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use serde::{Deserialize, Serialize};

use crate::domain::webrtc::peer::value_object::PeerId;

// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
// これらは単なるパラメータであり、値自体のvalidationはskyway-webrtc-gateway-api crate内で行われる

/// Event fired when a call request is received
pub use skyway_webrtc_gateway_api::peer::PeerCallEvent;
/// Event fired when a peer object is closed
pub use skyway_webrtc_gateway_api::peer::PeerCloseEvent;
/// Event fired when a data connection request is received
pub use skyway_webrtc_gateway_api::peer::PeerConnectionEvent;
/// Event fired when an error occurs
pub use skyway_webrtc_gateway_api::peer::PeerErrorEvent;
/// Enum represents PeerEvents
pub use skyway_webrtc_gateway_api::peer::PeerEventEnum;
/// Event fired when a peer object is opened
pub use skyway_webrtc_gateway_api::peer::PeerOpenEvent;
/// Shows PeerStatus
pub use skyway_webrtc_gateway_api::peer::PeerStatusMessage;

// POST /peerで必要なパラメータ類

/// Parameter for POST /peer
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}
