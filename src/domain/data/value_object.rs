use serde::Deserialize;

pub use skyway_webrtc_gateway_api::data::{
    ConnectQuery, DataConnectionId, DataConnectionIdWrapper, DataId,
};

#[derive(Deserialize, Debug)]
pub(crate) struct DataIdWrapper {
    pub data_id: DataId,
}
