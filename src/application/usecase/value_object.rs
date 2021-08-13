use std::sync::Arc;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use shaku::HasComponent;
use skyway_webrtc_gateway_api::media::RtcpId;

use crate::application::usecase::service::{EventListener, Service};
use crate::domain::common::value_object::SocketInfo;
use crate::domain::data::value_object::{DataConnectionIdWrapper, DataId};
use crate::domain::media::value_object::{
    AnswerResult, MediaConnectionEventEnum, MediaConnectionIdWrapper, MediaId,
};
use crate::domain::media::value_object::{MediaIdWrapper, RtcpIdWrapper};
use crate::domain::peer::value_object::{PeerEventEnum, PeerInfo};
use crate::prelude::DataConnectionEventEnum;

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "command")]
pub enum PeerServiceParams {
    #[serde(rename = "CREATE")]
    Create { params: Value },
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
    use crate::domain::peer::value_object::CreatePeerParams;
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
            let module = PeerCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        PeerServiceParams::Delete { params } => {
            let module = PeerDeleteServiceContainer::builder().build();
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
#[serde(untagged)]
pub enum ResponseMessageBodyEnum {
    PeerCreate(PeerInfo),
    PeerDelete(PeerInfo),
    PeerEvent(PeerEventEnum),
    DataCreate(SocketInfo<DataId>),
    DataConnect(DataConnectionIdWrapper),
    DataDelete(DataId),
    DataDisconnect(DataConnectionIdWrapper),
    DataRedirect(DataConnectionIdWrapper),
    DataEvent(DataConnectionEventEnum),
    MediaContentCreate(SocketInfo<MediaId>),
    MediaContentDelete(MediaIdWrapper),
    MediaRtcpCreate(SocketInfo<RtcpId>),
    MediaRtcpDelete(RtcpIdWrapper),
    MediaCall(MediaConnectionIdWrapper),
    MediaAnswer(AnswerResult),
    MediaEvent(MediaConnectionEventEnum),
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

// FIXME: no test
pub(crate) fn event_factory(
    message: ResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    fn value<V: Serialize, T: HasComponent<dyn EventListener>>(
        param: V,
        component: T,
    ) -> (Value, Arc<dyn EventListener>) {
        let value = serde_json::to_value(&param).unwrap();
        (value, component.resolve())
    }

    match message {
        ResponseMessageBodyEnum::PeerCreate(params) => {
            let component = PeerEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        ResponseMessageBodyEnum::DataConnect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        ResponseMessageBodyEnum::DataRedirect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        ResponseMessageBodyEnum::MediaCall(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        ResponseMessageBodyEnum::MediaAnswer(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}
