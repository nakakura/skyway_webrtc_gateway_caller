use crate::application::dto::request_message::ServiceParams;
use crate::{error, ResponseResult};

pub async fn format_input_json(json_str: &str) -> Result<ServiceParams, error::Error> {
    serde_json::from_str::<ServiceParams>(json_str).map_err(|e| {
        let message = format!("Presentation layer received invalid json {:?}", e);
        error::Error::create_local_error(&message)
    })
}

#[cfg(test)]
mod format_input_json_test {
    use super::*;

    #[tokio::test]
    async fn format_valid_json() {
        let json = r#"{
        "type": "PEER",
        "command": "CREATE",
        "params": {
            "key": "api_key",
            "domain": "localhost",
            "peer_id": "my_peer",
            "turn": true
        }
    }"#;

        let message = format_input_json(json).await;
        assert!(message.is_ok());
    }

    #[tokio::test]
    async fn format_invalid_json() {
        let json = r#"{
        "params": {
            "peer_id": "peer_id",
            "token": "pt-9749250e-d157-4f80-9ee2-359ce8524308"
        }
    }"#;

        let message = format_input_json(json).await;
        assert!(message.is_err());
    }
}

pub fn serialize_service_params(params: &ResponseResult) -> String {
    serde_json::to_string(params).unwrap()
}
