pub(crate) mod usecase;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub(crate) mod service_creator {
    // 何故かwarningが出るのでマクロを入れる
    #[allow(unused_imports)]
    use serde_json::Value;

    #[allow(unused_imports)]
    use crate::application::usecase::service::execute_service;
    #[allow(unused_imports)]
    use crate::application::usecase::service::Service;
    #[allow(unused_imports)]
    use crate::application::usecase::value_object::ResponseMessage;
    #[allow(unused_imports)]
    use crate::application::usecase::value_object::ServiceParams;

    pub(crate) async fn create(params: ServiceParams) -> ResponseMessage {
        use shaku::HasComponent;

        use crate::di::*;

        // FIXME: 同じ内容の重複
        match params {
            ServiceParams::PeerCreate { params } => {
                let module = PeerCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::PeerDelete { params } => {
                let module = PeerDeleteServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::DataCreate { params } => {
                let module = DataCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::DataDelete { params } => {
                let module = DataDeleteServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::DataConnect { params } => {
                let module = DataConnectServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::DataRedirect { params } => {
                let module = DataRedirectServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::MediaContentCreate { params } => {
                let module = MediaContentCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::MediaRtcpCreate { params: _ } => {
                let module = MediaRtcpCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, Value::Null).await
            }
            ServiceParams::MediaCall { params } => {
                let module = MediaCallServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            ServiceParams::MediaAnswer { params } => {
                let module = MediaAnswerServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                execute_service(service, params).await
            }
            _ => {
                unreachable!()
            }
        }
    }
}
