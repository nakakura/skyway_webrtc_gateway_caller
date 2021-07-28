use serde::{Deserialize, Serialize};
pub use skyway_webrtc_gateway_api::media::{
    AnswerQuery, AnswerResponse, AnswerResponseParams, CallQuery, CallResponse, Constraints,
    MediaConnectionEventEnum, MediaConnectionId, MediaConnectionIdWrapper, MediaConnectionStatus,
    MediaId, MediaParams, RedirectParameters, RtcpId, SsrcPair,
};

#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug, Clone)]
pub struct MediaIdWrapper {
    pub media_id: MediaId,
}

#[derive(Serialize, Deserialize, PartialOrd, PartialEq, Debug, Clone)]
pub struct RtcpIdWrapper {
    pub rtcp_id: RtcpId,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct AnswerResult {
    pub media_connection_id: MediaConnectionId,
    pub send_sockets: Option<AnswerResponseParams>,
    pub recv_sockets: Option<RedirectParameters>,
}
