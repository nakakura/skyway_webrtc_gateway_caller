// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

use serde::{Deserialize, Serialize};

use crate::prelude::DataConnectionId;
use crate::prelude::PhantomId;
use crate::prelude::SocketInfo;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。
pub use skyway_webrtc_gateway_api::data::{
    ConnectQuery, DataConnectionEventEnum, DataConnectionIdWrapper, DataConnectionStatus,
    DataIdWrapper, RedirectDataParams, RedirectDataResponse,
};

// JSON Parse用の定義
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RedirectParams {
    pub data_connection_id: DataConnectionId,
    pub feed_params: Option<DataIdWrapper>,
    pub redirect_params: Option<SocketInfo<PhantomId>>,
}
