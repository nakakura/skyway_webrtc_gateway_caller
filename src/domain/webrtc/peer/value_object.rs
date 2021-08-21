// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::*;

use crate::error;

#[cfg(test)]
use mockall::automock;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
pub use skyway_webrtc_gateway_api::peer::{
    PeerCallEvent, PeerCloseEvent, PeerConnectionEvent, PeerErrorEvent, PeerEventEnum, PeerId,
    PeerInfo, PeerOpenEvent, Token,
};

/// POST /peerで必要なパラメータ類
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}

// FIXME: Value Objectではない
#[cfg_attr(test, automock)]
#[async_trait]
pub trait PeerApi: Interface {
    async fn event(&self, peer_info: PeerInfo) -> Result<PeerEventEnum, error::Error>;
}

// FIXME: Value Objectではない
#[cfg_attr(test, automock)]
#[async_trait]
pub trait Peer: Interface {
    async fn event(&self, message: Value) -> Result<PeerEventEnum, error::Error>;
}

#[derive(Component)]
#[shaku(interface = Peer)]
pub(crate) struct PeerImpl {
    #[shaku(inject)]
    api: Arc<dyn PeerApi>,
}

#[async_trait]
impl Peer for PeerImpl {
    async fn event(&self, message: Value) -> Result<PeerEventEnum, error::Error> {
        // ドメイン層の知識として、JSONメッセージのParseを行う
        let peer_info = serde_json::from_value::<PeerInfo>(message)
            .map_err(|e| error::Error::SerdeError { error: e })?;
        self.api.event(peer_info).await
    }
}
