use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error;

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
    use crate::application::dto::{PeerServiceParams, ServiceParams};
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
