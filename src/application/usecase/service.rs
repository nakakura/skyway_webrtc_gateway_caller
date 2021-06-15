use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;
use tokio::sync::mpsc::Sender;

use crate::application::usecase::data::create::CreateDataSuccessMessage;
use crate::application::usecase::data::delete::DeleteDataSuccessMessage;
use crate::application::usecase::peer::create::CreatePeerSuccessMessage;
use crate::application::usecase::peer::delete::DeletePeerSuccessMessage;
use crate::application::usecase::peer::event::PeerEventMessage;
use crate::application::usecase::ErrorMessage;

// 副作用のない単発のサービス
// WebRTC Gatewayを叩いて結果を返す
// create系のように、createのapiを叩いたあとopenイベントを確認するためevent apiを叩くものもあるが、
// return以外の結果の外部出力やステータスを持たない
#[cfg_attr(test, automock)]
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
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum ServiceParams {
    PEER_CREATE { params: Value },
    PEER_DELETE { params: Value },
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq)]
#[serde(untagged)]
pub enum ReturnMessage {
    PEER_CREATE(CreatePeerSuccessMessage),
    PEER_DELETE(DeletePeerSuccessMessage),
    PEER_EVENT(PeerEventMessage),
    DATA_CREATE(CreateDataSuccessMessage),
    DATA_DELETE(DeleteDataSuccessMessage),
    ERROR(ErrorMessage),
}

#[cfg(test)]
mod serialize_enum {
    use crate::application::usecase::peer::create::CREATE_PEER_COMMAND;
    use crate::domain::peer::value_object::PeerInfo;

    use super::*;

    #[test]
    fn create_message() {
        let expected = "{\"result\":true,\"command\":\"PEER_CREATE\",\"params\":{\"peer_id\":\"peer_id\",\"token\":\"pt-9749250e-d157-4f80-9ee2-359ce8524308\"}}";

        // create a param
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let obj = CreatePeerSuccessMessage {
            result: true,
            command: CREATE_PEER_COMMAND.into(),
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
            command: CREATE_PEER_COMMAND.into(),
            error_message: "error".to_string(),
        };
        let ret_message = ReturnMessage::ERROR(obj);

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
    async fn execute(&self, event_tx: Sender<ReturnMessage>, params: Value) -> ReturnMessage;
    fn create_return_message(&self, result: Result<ReturnMessage, error::Error>) -> ReturnMessage {
        match result {
            Ok(message) => message,
            Err(e) => {
                let message = format!("{:?}", e);
                ReturnMessage::ERROR(ErrorMessage {
                    result: false,
                    command: self.command().into(),
                    error_message: message,
                })
            }
        }
    }
}
