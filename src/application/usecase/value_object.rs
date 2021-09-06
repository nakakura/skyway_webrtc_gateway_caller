use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::domain::webrtc::data::entity::{
    DataConnectionIdWrapper, DataConnectionStatus, DataSocket,
};
use crate::domain::webrtc::media::entity::{
    AnswerResult, MediaConnectionEventEnum, MediaConnectionIdWrapper, MediaConnectionStatus,
    MediaSocket, RtcpSocket,
};
use crate::domain::webrtc::media::entity::{MediaIdWrapper, RtcpIdWrapper};
use crate::domain::webrtc::peer::entity::PeerEventEnum;
use crate::domain::webrtc::peer::entity::PeerStatusMessage;
use crate::domain::webrtc::peer::value_object::PeerInfo;
use crate::prelude::{DataConnectionEventEnum, DataIdWrapper};

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum PeerServiceParams {
    #[serde(rename = "CREATE")]
    Create { params: Value },
    #[serde(rename = "STATUS")]
    Status { params: Value },
    #[serde(rename = "DELETE")]
    Delete { params: Value },
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum DataServiceParams {
    #[serde(rename = "CREATE")]
    Create { params: Value },
    #[serde(rename = "DELETE")]
    Delete { params: Value },
    #[serde(rename = "CONNECT")]
    Connect { params: Value },
    #[serde(rename = "REDIRECT")]
    Redirect { params: Value },
    #[serde(rename = "DISCONNECT")]
    Disconnect { params: Value },
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum MediaServiceParams {
    #[serde(rename = "CONTENT_CREATE")]
    ContentCreate { params: Value },
    #[serde(rename = "CONTENT_DELETE")]
    ContentDelete { params: Value },
    #[serde(rename = "RTCP_CREATE")]
    RtcpCreate { params: Option<Value> },
    #[serde(rename = "CALL")]
    Call { params: Value },
    #[serde(rename = "ANSWER")]
    Answer { params: Value },
}

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで受け取る
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ServiceParams {
    #[serde(rename = "PEER")]
    Peer(PeerServiceParams),
    #[serde(rename = "DATA")]
    Data(DataServiceParams),
    #[serde(rename = "MEDIA")]
    Media(MediaServiceParams),
}

#[cfg(test)]
mod service_params_deserialize {
    use crate::application::usecase::value_object::{PeerServiceParams, ServiceParams};
    use crate::domain::webrtc::peer::entity::CreatePeerParams;
    use crate::prelude::PeerInfo;

    #[test]
    fn create_message() {
        let message = r#"{
            "type": "PEER",
            "command": "CREATE",
            "params": {
                "base_url": "http://localhost:8000",
                "key": "api_key",
                "domain": "localhost",
                "peer_id": "peer_id",
                "turn": true
            }
        }"#;

        let create_message = serde_json::from_str::<ServiceParams>(message);
        if let Ok(ServiceParams::Peer(PeerServiceParams::Create { params })) = create_message {
            let _ = serde_json::from_value::<CreatePeerParams>(params).unwrap();
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn delete_message() {
        let message = r#"{
            "type": "PEER",
            "command": "DELETE",
            "params": {
                "peer_id": "my_peer_id",
                "token": "pt-9749250e-d157-4f80-9ee2-359ce8524308"
             }
        }"#;

        let create_message = serde_json::from_str::<ServiceParams>(message);
        if let Ok(ServiceParams::Peer(PeerServiceParams::Delete { params })) = create_message {
            let _ = serde_json::from_value::<PeerInfo>(params).unwrap();
            assert!(true);
        } else {
            assert!(false);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum PeerResponseMessageBodyEnum {
    #[serde(rename = "CREATE")]
    Create(PeerInfo),
    #[serde(rename = "STATUS")]
    Status(PeerStatusMessage),
    #[serde(rename = "DELETE")]
    Delete(PeerInfo),
    #[serde(rename = "EVENT")]
    Event(PeerEventEnum),
}

impl PeerResponseMessageBodyEnum {
    pub fn create_response_message(self) -> ResponseMessage {
        ResponseMessage::Success(ResponseMessageBodyEnum::Peer(self))
    }
}

#[test]
fn peer_response_message_body_enum_create_response_message() {
    let peer_id =
        PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
    let body_enum = PeerResponseMessageBodyEnum::Create(peer_id);
    let response_message = body_enum.create_response_message();
    // 型システムによって守られているので、ミスの発生しうる余地はErrorでのラップのみである
    if let ResponseMessage::Error(_) = response_message {
        assert!(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum DataResponseMessageBodyEnum {
    #[serde(rename = "CREATE")]
    Create(DataSocket),
    #[serde(rename = "CONNECT")]
    Connect(DataConnectionIdWrapper),
    #[serde(rename = "DELETE")]
    Delete(DataIdWrapper),
    #[serde(rename = "DISCONNECT")]
    Disconnect(DataConnectionIdWrapper),
    #[serde(rename = "REDIRECT")]
    Redirect(DataConnectionIdWrapper),
    #[serde(rename = "EVENT")]
    Event(DataConnectionEventEnum),
    #[serde(rename = "STATUS")]
    Status(DataConnectionStatus),
}

impl DataResponseMessageBodyEnum {
    pub fn create_response_message(self) -> ResponseMessage {
        ResponseMessage::Success(ResponseMessageBodyEnum::Data(self))
    }
}

#[test]
fn data_response_message_body_enum_create_response_message() {
    use skyway_webrtc_gateway_api::prelude::SerializableId;

    use crate::domain::webrtc::data::value_object::DataId;

    let data_id = DataId::try_create("da-4d053831-5dc2-461b-a358-d062d6115216").unwrap();
    let body_enum = DataResponseMessageBodyEnum::Delete(DataIdWrapper { data_id });
    let response_message = body_enum.create_response_message();
    // 型システムによって守られているので、ミスの発生しうる余地はErrorでのラップのみである
    if let ResponseMessage::Error(_) = response_message {
        assert!(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum MediaResponseMessageBodyEnum {
    #[serde(rename = "CONTENT_CREATE")]
    ContentCreate(MediaSocket),
    #[serde(rename = "CONTENT_DELETE")]
    ContentDelete(MediaIdWrapper),
    #[serde(rename = "RTCP_CREATE")]
    RtcpCreate(RtcpSocket),
    #[serde(rename = "RTCP_DELETE")]
    RtcpDelete(RtcpIdWrapper),
    #[serde(rename = "CALL")]
    Call(MediaConnectionIdWrapper),
    #[serde(rename = "ANSWER")]
    Answer(AnswerResult),
    #[serde(rename = "EVENT")]
    Event(MediaConnectionEventEnum),
    #[serde(rename = "STATUS")]
    Status(MediaConnectionStatus),
}

impl MediaResponseMessageBodyEnum {
    pub fn create_response_message(self) -> ResponseMessage {
        ResponseMessage::Success(ResponseMessageBodyEnum::Media(self))
    }
}

#[test]
fn media_response_message_body_enum_create_response_message() {
    use skyway_webrtc_gateway_api::prelude::SerializableId;

    use crate::domain::webrtc::media::value_object::MediaId;

    let media_id = MediaId::try_create("vi-4d053831-5dc2-461b-a358-d062d6115216").unwrap();
    let body_enum = MediaResponseMessageBodyEnum::ContentDelete(MediaIdWrapper { media_id });
    let response_message = body_enum.create_response_message();
    // 型システムによって守られているので、ミスの発生しうる余地はErrorでのラップのみである
    if let ResponseMessage::Error(_) = response_message {
        assert!(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ResponseMessageBodyEnum {
    #[serde(rename = "PEER")]
    Peer(PeerResponseMessageBodyEnum),
    #[serde(rename = "DATA")]
    Data(DataResponseMessageBodyEnum),
    #[serde(rename = "MEDIA")]
    Media(MediaResponseMessageBodyEnum),
}

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで渡せるように定義する
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ResponseMessage {
    Success(ResponseMessageBodyEnum),
    Error(String),
}

impl Serialize for ResponseMessage {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Person", 2)?;
        match self {
            ResponseMessage::Success(value) => {
                state.serialize_field("is_success", &true)?;
                state.serialize_field("result", &value)?;
            }
            ResponseMessage::Error(value) => {
                state.serialize_field("is_success", &false)?;
                state.serialize_field("result", &value)?;
            }
        }
        state.end()
    }
}

#[cfg(test)]
mod response_message_serialize {
    use serde_json::Value;

    use crate::application::usecase::value_object::{PeerResponseMessageBodyEnum, ResponseMessage};
    use crate::domain::webrtc::peer::value_object::PeerInfo;
    use crate::prelude::ResponseMessageBodyEnum;

    #[test]
    fn create_message() {
        let expected = serde_json::from_str::<Value>("{\"is_success\":true,\"result\":{\"peer_id\":\"peer_id\",\"token\":\"pt-9749250e-d157-4f80-9ee2-359ce8524308\", \"type\": \"PEER\", \"command\": \"CREATE\"}}");

        // create a param
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let ret_message = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
            PeerResponseMessageBodyEnum::Create(peer_info),
        ));

        // serialize
        let message = serde_json::to_string(&ret_message).unwrap();

        //evaluate
        assert_eq!(
            expected.unwrap(),
            serde_json::from_str::<Value>(&message).unwrap(),
        );
    }
}
