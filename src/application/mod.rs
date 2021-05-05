use std::sync::Arc;

use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shaku::Interface;
use skyway_webrtc_gateway_api::error;

use usecase::peer::create::{CreatePeerSuccessMessage, ErrorMessage};
use usecase::peer::delete::DeletePeerSuccessMessage;
use usecase::peer::event::PeerEventMessage;
use usecase::service::{Service, ServiceParams};

pub(crate) mod usecase;

// FIXME: 未テスト
// Unit TestではなくIntegration Testでテストするため
#[cfg_attr(test, automock)]
pub(crate) mod service_creator {
    use shaku::HasComponent;

    use crate::application::usecase::service::{ReturnMessage, Service, ServiceParams};
    use crate::di::*;

    use super::*;

    pub(crate) async fn create(params_string: String) -> ReturnMessage {
        let params = serde_json::from_str::<ServiceParams>(&params_string);

        match params {
            Ok(ServiceParams::PEER_CREATE { params }) => {
                let module = PeerCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                service.execute(params).await
            }
            Ok(ServiceParams::PEER_DELETE { params }) => {
                let module = PeerDeleteServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                service.execute(params).await
            }
            Err(e) => ReturnMessage::ERROR(ErrorMessage {
                result: false,
                command: "UNKNOWN".into(),
                error_message: format!("{:?}", e),
            }),
        }
    }
}
