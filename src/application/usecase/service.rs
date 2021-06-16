use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;
use tokio::sync::mpsc::Sender;

use crate::application::usecase::data::delete::DataDeleteResponseMessage;
use crate::application::usecase::data::disconnect::DataDisconnectSuccessMessage;
use crate::application::usecase::ErrorMessage;
use crate::PeerInfo;

// 副作用のない単発のサービス
// WebRTC Gatewayを叩いて結果を返す
// create系のように、createのapiを叩いたあとopenイベントを確認するためevent apiを叩くものもあるが、
// return以外の結果の外部出力やステータスを持たない
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait Service: Interface {
    fn command(&self) -> &'static str;
    async fn execute(&self, params: Value) -> ResponseMessage;
    fn create_return_message(
        &self,
        result: Result<ResponseMessage, error::Error>,
    ) -> ResponseMessage {
        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ResponseMessage::ERROR(ErrorMessage {
                    result: false,
                    command: self.command().into(),
                    error_message: message,
                })
            }
        }
    }
}

// JSONでクライアントから受け取るメッセージ
// JSONとしてなので、キャメルケースではなくスネークケースで受け取る
#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum ServiceParams {
    #[serde(rename = "PEER_CREATE")]
    PeerCreate {
        params: Value,
    },
    PEER_DELETE {
        params: Value,
    },
    DATA_CREATE {
        params: Value,
    },
    DATA_DELETE {
        params: Value,
    },
    DATA_CONNECT {
        params: Value,
    },
    DATA_DISCONNECT {
        params: Value,
    },
}

#[cfg(test)]
mod deserialize_str {
    use skyway_webrtc_gateway_api::peer::PeerInfo;

    use crate::domain::peer::value_object::CreatePeerParams;

    use super::*;

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
        if let Ok(ServiceParams::PEER_DELETE { params }) = create_message {
            let _ = serde_json::from_value::<PeerInfo>(params).unwrap();
            assert!(true);
        } else {
            assert!(false);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ErrorMessageRefactor {
    is_success: bool,
    pub result: String,
}

impl ErrorMessageRefactor {
    pub fn new(result: String) -> Self {
        Self {
            is_success: false,
            result,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ResponseMessageContent<T: Serialize + PartialEq> {
    is_success: bool,
    pub result: T,
}

impl<T: Serialize + PartialEq> ResponseMessageContent<T> {
    pub fn new(result: T) -> Self {
        Self {
            is_success: true,
            result,
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
    DATA_DISCONNECT(DataDisconnectSuccessMessage),
    ERROR(ErrorMessage),
}

#[cfg(test)]
mod serialize_enum {
    use crate::domain::peer::value_object::PeerInfo;

    use super::*;

    #[test]
    fn create_message() {
        let expected = "{\"is_success\":true,\"result\":{\"peer_id\":\"peer_id\",\"token\":\"pt-9749250e-d157-4f80-9ee2-359ce8524308\"}}";

        // create a param
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let content = ResponseMessageContent::new(peer_info);
        use crate::application::usecase::peer::create::PeerCreateResponseMessage;
        let ret_message = ResponseMessage::PeerCreate(PeerCreateResponseMessage::Success(content));

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
            command: "PEER_CREATE".into(),
            error_message: "error".to_string(),
        };
        let ret_message = ResponseMessage::ERROR(obj);

        // serialize
        let message = serde_json::to_string(&ret_message).unwrap();

        //evaluate
        assert_eq!(message.as_str(), expected);
    }
}

// WebRTC Gatewayのイベントを監視する
// Errorの発生もしくはCLOSEイベントの発火まで監視し続ける
// 終了理由をreturnする
// 個別の取得したイベントについては、TIMEOUTを除きexecuteメソッドで受け取ったSenderで通知する
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait EventListener: Interface {
    fn command(&self) -> &'static str;
    async fn execute(&self, event_tx: Sender<ResponseMessage>, params: Value) -> ResponseMessage;
    fn create_return_message(
        &self,
        result: Result<ResponseMessage, error::Error>,
    ) -> ResponseMessage {
        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ResponseMessage::ERROR(ErrorMessage {
                    result: false,
                    command: self.command().into(),
                    error_message: message,
                })
            }
        }
    }
}
