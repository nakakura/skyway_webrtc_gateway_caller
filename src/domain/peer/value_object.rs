use serde::{Deserialize, Serialize};

pub use skyway_webrtc_gateway_api::peer::PeerEventEnum;
pub use skyway_webrtc_gateway_api::prelude::PeerId;
pub use skyway_webrtc_gateway_api::prelude::PeerInfo;
pub use skyway_webrtc_gateway_api::prelude::Token;

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub base_url: String,
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}
