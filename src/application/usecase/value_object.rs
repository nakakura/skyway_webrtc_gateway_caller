use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use shaku::HasComponent;
use skyway_webrtc_gateway_api::media::RtcpId;

use crate::application::usecase::service::{EventListener, Service};
use crate::domain::webrtc::common::value_object::SocketInfo;
use crate::domain::webrtc::data::value_object::{DataConnectionIdWrapper, DataId};
use crate::domain::webrtc::media::value_object::{
    AnswerResult, MediaConnectionEventEnum, MediaConnectionIdWrapper, MediaSocket,
};
use crate::domain::webrtc::media::value_object::{MediaIdWrapper, RtcpIdWrapper};
use crate::domain::webrtc::peer::value_object::PeerStatusMessage;
use crate::domain::webrtc::peer::value_object::{PeerEventEnum, PeerInfo};
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
    use crate::domain::webrtc::peer::value_object::CreatePeerParams;
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

fn peer_service_factory(params: PeerServiceParams) -> (Value, Arc<dyn Service>) {
    use crate::di::*;

    match params {
        PeerServiceParams::Create { params } => {
            let module = PeerCreateServiceRefactorContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        PeerServiceParams::Status { params } => {
            let module = PeerStatusServiceRefactorContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        PeerServiceParams::Delete { params } => {
            let module = PeerDeleteServiceRefactorContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
    }
}

fn data_service_factory(params: DataServiceParams) -> (Value, Arc<dyn Service>) {
    use crate::di::*;

    match params {
        DataServiceParams::Create { params } => {
            let module = DataCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        DataServiceParams::Delete { params } => {
            let module = DataDeleteServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        DataServiceParams::Connect { params } => {
            let module = DataConnectServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        DataServiceParams::Redirect { params } => {
            let module = DataRedirectServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        _ => unreachable!(),
    }
}

fn media_service_factory(params: MediaServiceParams) -> (Value, Arc<dyn Service>) {
    use crate::di::*;

    match params {
        MediaServiceParams::ContentCreate { params } => {
            let module = MediaContentCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        MediaServiceParams::RtcpCreate { params: _ } => {
            let module = MediaRtcpCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (Value::Null, service)
        }
        MediaServiceParams::Call { params } => {
            let module = MediaCallServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        MediaServiceParams::Answer { params } => {
            let module = MediaAnswerServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        _ => unreachable!(),
    }
}

// FIXME: no unit test
pub(crate) fn service_factory(params: ServiceParams) -> (Value, Arc<dyn Service>) {
    match params {
        ServiceParams::Peer(params) => peer_service_factory(params),
        ServiceParams::Data(params) => data_service_factory(params),
        ServiceParams::Media(params) => media_service_factory(params),
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
    Create(SocketInfo<DataId>),
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
}

impl DataResponseMessageBodyEnum {
    pub fn create_response_message(self) -> ResponseMessage {
        ResponseMessage::Success(ResponseMessageBodyEnum::Data(self))
    }
}

#[test]
fn data_response_message_body_enum_create_response_message() {
    use skyway_webrtc_gateway_api::prelude::SerializableId;

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
    RtcpCreate(SocketInfo<RtcpId>),
    #[serde(rename = "RTCP_DELETE")]
    RtcpDelete(RtcpIdWrapper),
    #[serde(rename = "CALL")]
    Call(MediaConnectionIdWrapper),
    #[serde(rename = "ANSWER")]
    Answer(AnswerResult),
    #[serde(rename = "EVENT")]
    Event(MediaConnectionEventEnum),
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

fn value<V: Serialize, T: HasComponent<dyn EventListener>>(
    param: V,
    component: T,
) -> (Value, Arc<dyn EventListener>) {
    // paramsはserializeをimplementしているので、エラーが出ることはなく、unwrapで問題ない
    let value = serde_json::to_value(&param).unwrap();
    (value, component.resolve())
}

fn peer_event_factory(
    params: PeerResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        PeerResponseMessageBodyEnum::Create(params) => {
            let component = PeerEventServiceRefactorContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

fn data_event_factory(
    params: DataResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        DataResponseMessageBodyEnum::Connect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        DataResponseMessageBodyEnum::Redirect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

fn media_event_factory(
    params: MediaResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        MediaResponseMessageBodyEnum::Call(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        MediaResponseMessageBodyEnum::Answer(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

// FIXME: no test
pub(crate) fn event_factory(
    message: ResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    match message {
        ResponseMessageBodyEnum::Peer(params) => peer_event_factory(params),
        ResponseMessageBodyEnum::Data(params) => data_event_factory(params),
        ResponseMessageBodyEnum::Media(params) => media_event_factory(params),
    }
}
