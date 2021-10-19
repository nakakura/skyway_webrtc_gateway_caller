pub mod request_message {
    use serde::de::DeserializeOwned;
    use serde::{Deserialize, Serialize};
    use skyway_webrtc_gateway_api::error;

    // ユーザから与えられたJSONをDTOとしてラップする
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct Parameter(pub serde_json::Value);

    impl Parameter {
        pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, error::Error> {
            serde_json::from_value::<T>(self.0).map_err(|e| error::Error::SerdeError { error: e })
        }
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "command")]
    pub enum PeerServiceParams {
        #[serde(rename = "CREATE")]
        Create { params: Parameter },
        #[serde(rename = "STATUS")]
        Status { params: Parameter },
        #[serde(rename = "DELETE")]
        Delete { params: Parameter },
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "command")]
    pub enum DataServiceParams {
        #[serde(rename = "CREATE")]
        Create { params: Parameter },
        #[serde(rename = "DELETE")]
        Delete { params: Parameter },
        #[serde(rename = "CONNECT")]
        Connect { params: Parameter },
        #[serde(rename = "REDIRECT")]
        Redirect { params: Parameter },
        #[serde(rename = "DISCONNECT")]
        Disconnect { params: Parameter },
    }

    #[allow(non_camel_case_types)]
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    #[serde(tag = "command")]
    pub enum MediaServiceParams {
        #[serde(rename = "CONTENT_CREATE")]
        ContentCreate { params: Parameter },
        #[serde(rename = "CONTENT_DELETE")]
        ContentDelete { params: Parameter },
        #[serde(rename = "RTCP_CREATE")]
        RtcpCreate { params: Option<Parameter> },
        #[serde(rename = "CALL")]
        Call { params: Parameter },
        #[serde(rename = "ANSWER")]
        Answer { params: Parameter },
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
        use crate::application::dto::request_message::{PeerServiceParams, ServiceParams};
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
                let _ = serde_json::from_value::<CreatePeerParams>(params.0).unwrap();
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
                let _ = serde_json::from_value::<PeerInfo>(params.0).unwrap();
                assert!(true);
            } else {
                assert!(false);
            }
        }
    }
}

pub mod response_message {
    use serde::ser::SerializeStruct;
    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Value;

    use crate::domain::webrtc::common::value_object::SocketInfo;
    use crate::domain::webrtc::data::entity::{
        DataConnectionEventEnum, DataConnectionIdWrapper, DataConnectionStatus, DataIdWrapper,
    };
    use crate::domain::webrtc::data::value_object::DataId;
    use crate::domain::webrtc::media::entity::{
        AnswerResult, MediaConnectionEventEnum, MediaConnectionIdWrapper, MediaConnectionStatus,
        MediaIdWrapper, MediaSocket, RtcpIdWrapper, RtcpSocket,
    };
    use crate::domain::webrtc::peer::entity::PeerStatusMessage;
    use crate::error;
    use crate::{PeerEventEnum, PeerInfo};

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

        use crate::domain::webrtc::data::entity::DataIdWrapper;
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

        use crate::domain::webrtc::media::entity::MediaIdWrapper;
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
    #[derive(Debug, Clone, PartialEq)]
    pub enum ResponseMessage {
        Success(ResponseMessageBodyEnum),
        Error(String),
    }

    impl ResponseMessage {
        pub fn from_str(json: &str) -> Result<ResponseMessage, error::Error> {
            #[derive(Deserialize)]
            struct ResponseMessageStruct {
                is_success: bool,
                result: Value,
            }
            let value = serde_json::from_str::<ResponseMessageStruct>(json)
                .map_err(|e| error::Error::SerdeError { error: e })?;
            match value.is_success {
                true => {
                    let content: ResponseMessageBodyEnum = serde_json::from_value(value.result)
                        .map_err(|e| error::Error::SerdeError { error: e })?;
                    Ok(ResponseMessage::Success(content))
                }
                _ => {
                    let content: String = serde_json::from_value(value.result)
                        .map_err(|e| error::Error::SerdeError { error: e })?;
                    Ok(ResponseMessage::Error(content))
                }
            }
        }
    }

    impl Serialize for ResponseMessage {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
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
    mod response_message_serialize_deserialize {
        use crate::application::dto::response_message::{
            PeerResponseMessageBodyEnum, ResponseMessage, ResponseMessageBodyEnum,
        };
        use crate::domain::webrtc::peer::value_object::PeerInfo;

        #[test]
        fn serialize_deserialize() {
            // create a param
            let peer_info =
                PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
            let ret_message = ResponseMessage::Success(ResponseMessageBodyEnum::Peer(
                PeerResponseMessageBodyEnum::Create(peer_info),
            ));

            // serialize
            let message = serde_json::to_string(&ret_message).unwrap();

            let result = ResponseMessage::from_str(&message).unwrap();

            //evaluate
            assert_eq!(result, ret_message);
        }
    }
}
