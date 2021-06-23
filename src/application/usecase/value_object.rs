use serde::{Deserialize, Serialize};
use serde_json::Value;

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

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで渡せるように定義する
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ResponseMessage {
    #[serde(rename = "PEER_CREATE")]
    PeerCreate(super::peer::create::PeerCreateResponseMessage),
    #[serde(rename = "PEER_DELETE")]
    PeerDelete(super::peer::delete::PeerDeleteResponseMessage),
    #[serde(rename = "PEER_EVENT")]
    PeerEvent(super::peer::event::PeerEventResponseMessage),
    #[serde(rename = "DATA_CREATE")]
    DataCreate(super::data::create::DataCreateResponseMessage),
    #[serde(rename = "DATA_DELETE")]
    DataDelete(super::data::delete::DataDeleteResponseMessage),
    #[serde(rename = "DATA_CONNECT")]
    DataConnect(super::data::connect::DataConnectResponseMessage),
    #[serde(rename = "DATA_DISCONNECT")]
    DataDisconnect(super::data::disconnect::DataDisconnectResponseMessage),
}

#[cfg(test)]
mod response_message_serialize {
    use crate::domain::peer::value_object::PeerInfo;

    #[test]
    fn create_message() {
        let expected = "{\"is_success\":true,\"result\":{\"peer_id\":\"peer_id\",\"token\":\"pt-9749250e-d157-4f80-9ee2-359ce8524308\"}}";

        // create a param
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let content = ResponseMessageBody::new(peer_info);
        use crate::application::usecase::peer::create::PeerCreateResponseMessage;
        use crate::application::usecase::value_object::{ResponseMessage, ResponseMessageBody};
        let ret_message = ResponseMessage::PeerCreate(PeerCreateResponseMessage::Success(content));

        // serialize
        let message = serde_json::to_string(&ret_message).unwrap();

        //evaluate
        assert_eq!(message.as_str(), expected);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorMessage {
    is_success: bool,
    pub result: String,
}

impl ErrorMessage {
    pub fn new(result: String) -> Self {
        Self {
            is_success: false,
            result,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResponseMessageBody<T: Serialize + PartialEq> {
    is_success: bool,
    pub result: T,
}

impl<T: Serialize + PartialEq> ResponseMessageBody<T> {
    pub fn new(result: T) -> Self {
        Self {
            is_success: true,
            result,
        }
    }
}
