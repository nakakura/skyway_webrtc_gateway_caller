// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use serde::{Deserialize, Serialize};

use crate::domain::webrtc::peer::value_object::PeerId;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
/// これらは単なるパラメータであり、値自体のvalidationはskyway-webrtc-gateway-api crate内で行われる
pub use skyway_webrtc_gateway_api::peer::{
    PeerCallEvent, PeerCloseEvent, PeerConnectionEvent, PeerErrorEvent, PeerEventEnum,
    PeerOpenEvent, PeerStatusMessage,
};

/// POST /peerで必要なパラメータ類
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}
