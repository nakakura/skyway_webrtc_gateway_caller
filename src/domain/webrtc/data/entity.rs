// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のentity, value_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

use serde::{Deserialize, Serialize};

use crate::domain::webrtc::common::value_object::{PhantomId, SocketInfo};

// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。

/// Parameter for POST /data/connections API
pub use skyway_webrtc_gateway_api::data::ConnectQuery;
/// Enum represents events of DataConnection
pub use skyway_webrtc_gateway_api::data::DataConnectionEventEnum;
/// Id associated with DataConnection
pub use skyway_webrtc_gateway_api::data::DataConnectionId;
/// Wrapper to adapt to JSON format
pub use skyway_webrtc_gateway_api::data::DataConnectionIdWrapper;
/// Shows status of DataConnection
pub use skyway_webrtc_gateway_api::data::DataConnectionStatus;
/// Id associated with Data
pub use skyway_webrtc_gateway_api::data::DataId;
/// Wrapper to adapt to JSON format
pub use skyway_webrtc_gateway_api::data::DataIdWrapper;
/// Parameter for PUT /data/connections API
pub use skyway_webrtc_gateway_api::data::RedirectDataParams;
/// Response of PUT /data/connections API
pub use skyway_webrtc_gateway_api::data::RedirectDataResponse;

// JSON Parse用の定義
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RedirectParams {
    pub data_connection_id: DataConnectionId,
    pub feed_params: Option<DataIdWrapper>,
    pub redirect_params: Option<SocketInfo<PhantomId>>,
}
