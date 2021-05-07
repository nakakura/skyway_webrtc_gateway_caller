#[cfg(test)]
use mockall::automock;

pub(crate) mod usecase;

// Unit TestではなくIntegration Testでテストする
#[cfg_attr(test, automock)]
pub(crate) mod service_creator {
    use crate::application::usecase::service::ReturnMessage;

    pub(crate) async fn create(params_string: String) -> ReturnMessage {
        use shaku::HasComponent;

        use crate::application::usecase::service::{Service, ServiceParams};
        use crate::di::*;

        match serde_json::from_str::<ServiceParams>(&params_string) {
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
            Err(e) => ReturnMessage::ERROR(crate::ErrorMessage {
                result: false,
                command: "UNKNOWN".into(),
                error_message: format!("{:?}", e),
            }),
        }
    }
}
