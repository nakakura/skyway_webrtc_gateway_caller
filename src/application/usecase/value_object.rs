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
    #[serde(rename = "DATA_REDIRECT")]
    DataRedirect { params: Value },
    #[serde(rename = "DATA_DISCONNECT")]
    DataDisconnect { params: Value },
    #[serde(rename = "MEDIA_CONTENT_CREATE")]
    MediaContentCreate { params: Value },
    #[serde(rename = "MEDIA_CONTENT_DELETE")]
    MediaContentDelete { params: Value },
    #[serde(rename = "MEDIA_RTCP_CREATE")]
    MediaRtcpCreate { params: Option<Value> },
    #[serde(rename = "MEDIA_CALL")]
    MediaCall { params: Value },
    #[serde(rename = "MEDIA_ANSWER")]
    MediaAnswer { params: Value },
}

#[cfg(test)]
mod service_params_deserialize {
    use crate::application::usecase::value_object::ServiceParams;
    use crate::domain::peer::value_object::CreatePeerParams;
    use crate::prelude::PeerInfo;

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

// FIXME: no test
pub(crate) fn service_factory(params: ServiceParams) -> (Value, Arc<dyn Service>) {
    use crate::di::*;

    match params {
        ServiceParams::PeerCreate { params } => {
            let module = PeerCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::PeerDelete { params } => {
            let module = PeerDeleteServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::DataCreate { params } => {
            let module = DataCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::DataDelete { params } => {
            let module = DataDeleteServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::DataConnect { params } => {
            let module = DataConnectServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::DataRedirect { params } => {
            let module = DataRedirectServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::MediaContentCreate { params } => {
            let module = MediaContentCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::MediaRtcpCreate { params: _ } => {
            let module = MediaRtcpCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (Value::Null, service)
        }
        ServiceParams::MediaCall { params } => {
            let module = MediaCallServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        ServiceParams::MediaAnswer { params } => {
            let module = MediaAnswerServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        _ => unreachable!(),
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
