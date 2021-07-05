pub(crate) mod usecase;

#[cfg(test)]
use mockall::automock;

use crate::domain::peer::value_object::Token;
use usecase::value_object::ServiceParams;

// TODO: まだtestでしか使っていない
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) enum EventEnum {
    System(Token),
    Json(ServiceParams),
}

#[cfg_attr(test, automock)]
pub(crate) mod service_creator {
    // 何故かwarningが出るのでマクロを入れる
    #[allow(unused_imports)]
    use crate::application::usecase::service::execute_service;
    #[allow(unused_imports)]
    use crate::application::usecase::service::Service;
    #[allow(unused_imports)]
    use crate::application::usecase::value_object::ResponseMessage;
    #[allow(unused_imports)]
    use crate::application::usecase::value_object::ServiceParams;

    // TODO: まだtestでしか使っていない
    #[allow(dead_code)]
    pub(crate) async fn create(params: ServiceParams) -> ResponseMessage {
        use shaku::HasComponent;

        use crate::di::*;

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
            _ => {
                unreachable!()
            }
        }
    }
}

#[cfg_attr(test, automock)]
pub(crate) mod event {
    use tokio::sync::mpsc;

    use crate::application::EventEnum;
    use crate::domain::peer::value_object::PeerInfo;

    // TODO: 実装
    // peer eventを監視し続ける
    // peer objectがcloseしたら(CLOSE eventを受け取ったら)終了して、fuse_txにEventEnum::Systemを通知
    #[allow(dead_code)]
    pub(crate) async fn event(
        _peer_info: PeerInfo,
        _event_tx: mpsc::Sender<String>,
        _fuse_tx: mpsc::Sender<EventEnum>,
    ) {
    }
}
