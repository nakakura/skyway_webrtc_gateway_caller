use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

use crate::usecase::peer::create::{CreatePeerSuccessMessage, ErrorMessage};
use crate::usecase::peer::delete::DeletePeerSuccessMessage;
use crate::usecase::peer::event::PeerEventMessage;

#[async_trait]
pub(crate) trait Service: Interface {
    fn command(&self) -> &'static str;
    async fn execute(&self, params: Value) -> ReturnMessage;
    fn create_return_message(&self, result: Result<ReturnMessage, error::Error>) -> ReturnMessage {
        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ReturnMessage::ERROR(ErrorMessage {
                    result: false,
                    command: self.command(),
                    error_message: message,
                })
            }
        }
    }
}

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで受け取る
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "command")]
pub(crate) enum ServiceParams {
    PEER_CREATE { params: Value },
    PEER_DELETE { params: Value },
}

#[cfg(test)]
mod deserialize_str {
    use super::*;
    use crate::domain::peer::value_object::CreatePeerParams;
    use skyway_webrtc_gateway_api::peer::PeerInfo;

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
        if let Ok(ServiceParams::PEER_CREATE { params }) = create_message {
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
        if let Ok(ServiceParams::PEER_DELETE { params }) = create_message {
            let _ = serde_json::from_value::<PeerInfo>(params).unwrap();
            assert!(true);
        } else {
            assert!(false);
        }
    }
}

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで渡せるように定義する
#[allow(non_camel_case_types)]
#[derive(Serialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(untagged)]
pub(crate) enum ReturnMessage {
    PEER_CREATE(CreatePeerSuccessMessage),
    PEER_DELETE(DeletePeerSuccessMessage),
    PEER_EVENT(PeerEventMessage),
    ERROR(ErrorMessage),
}

#[cfg(test)]
mod serialize_enum {
    use super::*;
    use crate::domain::peer::value_object::PeerInfo;
    use crate::usecase::peer::create::CREATE_PEER_COMMAND;

    #[test]
    fn create_message() {
        let expected = "{\"result\":true,\"command\":\"PEER_CREATE\",\"params\":{\"peer_id\":\"peer_id\",\"token\":\"pt-9749250e-d157-4f80-9ee2-359ce8524308\"}}";

        // create a param
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let obj = CreatePeerSuccessMessage {
            result: true,
            command: CREATE_PEER_COMMAND,
            params: peer_info,
        };
        let ret_message = ReturnMessage::PEER_CREATE(obj);

        // serialize
        let message = serde_json::to_string(&ret_message).unwrap();

        //evaluate
        assert_eq!(message.as_str(), expected);
    }

    #[test]
    fn error_message() {
        let expected = "{\"result\":false,\"command\":\"PEER_CREATE\",\"error_message\":\"error\"}";

        // create a param
        let obj = ErrorMessage {
            result: false,
            command: CREATE_PEER_COMMAND,
            error_message: "error".to_string(),
        };
        let ret_message = ReturnMessage::ERROR(obj);

        // serialize
        let message = serde_json::to_string(&ret_message).unwrap();

        //evaluate
        assert_eq!(message.as_str(), expected);
    }
}
