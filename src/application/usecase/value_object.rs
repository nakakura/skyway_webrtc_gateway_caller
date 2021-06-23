use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::value_object::{DataConnectionIdWrapper, DataId, DataIdWrapper};
use crate::domain::peer::value_object::{PeerEventEnum, PeerInfo};

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで受け取る
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum ServiceParams {
    #[serde(rename = "PEER_CREATE")]
    PeerCreate { params: Value },
    #[serde(rename = "PEER_DELETE")]
    PeerDelete { params: Value },
    #[serde(rename = "DATA_CREATE")]
    DataCreate { params: Value },
    #[serde(rename = "DATA_DELETE")]
    DataDelete { params: Value },
    #[serde(rename = "DATA_CONNECT")]
    DataConnect { params: Value },
    #[serde(rename = "DATA_DISCONNECT")]
    DataDisconnect { params: Value },
}

#[cfg(test)]
mod service_params_deserialize {
    use skyway_webrtc_gateway_api::peer::PeerInfo;

    use crate::application::usecase::value_object::ServiceParams;
    use crate::domain::peer::value_object::CreatePeerParams;

    #[test]
    fn create_message() {
        let message = r#"{
            "command": "PEER_CREATE",
            "params": {
                "base_url": "http://localhost:8000",
                "key": "api_key",
                "domain": "localhost",
                "peer_id": "peer_id",
                "turn": true
            }
        }"#;

        let create_message = serde_json::from_str::<ServiceParams>(message);
        if let Ok(ServiceParams::PeerCreate { params }) = create_message {
            let _ = serde_json::from_value::<CreatePeerParams>(params).unwrap();
            assert!(true);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn delete_message() {
        let message = r#"{
            "command": "PEER_DELETE",
            "params": {
                "peer_id": "my_peer_id",
                "token": "pt-9749250e-d157-4f80-9ee2-359ce8524308"
             }
        }"#;

        let create_message = serde_json::from_str::<ServiceParams>(message);
        if let Ok(ServiceParams::PeerDelete { params }) = create_message {
            let _ = serde_json::from_value::<PeerInfo>(params).unwrap();
            assert!(true);
        } else {
            assert!(false);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ResponseMessageBodyEnum {
    PeerCreate(PeerInfo),
    PeerDelete(PeerInfo),
    PeerEvent(PeerEventEnum),
    DataCreate(SocketInfo<DataId>),
    DataConnect(DataConnectionIdWrapper),
    DataDelete(DataId),
    DataDisconnect(DataConnectionIdWrapper),
    DataRedirect(DataIdWrapper),
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

    use crate::application::usecase::value_object::ResponseMessage;
    use crate::domain::peer::value_object::PeerInfo;
    use crate::prelude::ResponseMessageBodyEnum;

    #[test]
    fn create_message() {
        let expected = serde_json::from_str::<Value>("{\"is_success\":true,\"result\":{\"peer_id\":\"peer_id\",\"token\":\"pt-9749250e-d157-4f80-9ee2-359ce8524308\"}}");

        // create a param
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let ret_message = ResponseMessage::Success(ResponseMessageBodyEnum::PeerCreate(peer_info));

        // serialize
        let message = serde_json::to_string(&ret_message).unwrap();

        //evaluate
        assert_eq!(
            expected.unwrap(),
            serde_json::from_str::<Value>(&message).unwrap(),
        );
    }
}
