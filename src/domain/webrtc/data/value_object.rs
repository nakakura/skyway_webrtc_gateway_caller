// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use std::net::IpAddr;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::domain::webrtc::common::value_object::{SerializableSocket, SocketInfo};
use crate::domain::webrtc::data::service::DataApi;
use crate::error;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/data APIに関係するものを利用する。
pub use skyway_webrtc_gateway_api::data::{
    ConnectQuery, DataConnectionEventEnum, DataConnectionId, DataConnectionIdWrapper, DataId,
    DataIdWrapper, RedirectDataResponse,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct DataSocket(pub(crate) SocketInfo<DataId>);

//　これらの各メソッドは、application::data内のUnit Testで間接的にテストされている
impl DataSocket {
    pub async fn try_create(api: Arc<dyn DataApi>) -> Result<Self, error::Error> {
        let socket = api.create().await?;
        Ok(DataSocket(socket))
    }

    pub async fn try_delete(
        api: Arc<dyn DataApi>,
        data_id: DataId,
    ) -> Result<DataId, error::Error> {
        api.delete(data_id).await
    }

    pub fn get_id(&self) -> Option<DataId> {
        self.0.get_id()
    }

    pub fn ip(&self) -> IpAddr {
        self.0.ip()
    }

    pub fn port(&self) -> u16 {
        self.0.port()
    }
}
