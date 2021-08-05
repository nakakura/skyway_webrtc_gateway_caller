pub(crate) mod usecase;

use std::sync::Arc;

use serde_json::Value;

use crate::application::usecase::service::Service;
use crate::application::usecase::value_object::ResponseMessage;
use crate::application::usecase::value_object::ServiceParams;

fn factory(params: ServiceParams) -> (Value, Arc<dyn Service>) {
    use shaku::HasComponent;

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

pub(crate) async fn run(params: ServiceParams) -> ResponseMessage {
    let (params, service) = factory(params);
    crate::application::usecase::service::execute_service(service, params).await
}
