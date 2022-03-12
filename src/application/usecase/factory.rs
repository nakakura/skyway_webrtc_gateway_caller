use std::sync::Arc;

use serde::Serialize;
use shaku::HasComponent;

use crate::application::dto::request_message::{
    DataServiceParams, MediaServiceParams, Parameter, PeerServiceParams, ServiceParams,
};
use crate::application::dto::response_message::{
    DataResponse, MediaResponse, PeerResponse, ResponseMessage,
};
use crate::application::usecase::service::{EventListener, Service};

fn value<V: Serialize, T: HasComponent<dyn EventListener>>(
    param: V,
    component: T,
) -> (Parameter, Arc<dyn EventListener>) {
    // paramsはserializeをimplementしているので、エラーが出ることはなく、unwrapで問題ない
    let value = serde_json::to_value(&param).unwrap();
    (Parameter(value), component.resolve())
}

fn peer_event_factory(
    params: PeerResponse,
) -> Option<(Parameter, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        PeerResponse::Create(params) => {
            let component = PeerEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

fn data_event_factory(
    params: DataResponse,
) -> Option<(Parameter, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        DataResponse::Connect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        DataResponse::Redirect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

fn media_event_factory(
    params: MediaResponse,
) -> Option<(Parameter, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        MediaResponse::Call(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        MediaResponse::Answer(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

// FIXME: no test
pub(crate) fn event_factory(
    message: ResponseMessage,
) -> Option<(Parameter, std::sync::Arc<dyn EventListener>)> {
    match message {
        ResponseMessage::Peer(params) => peer_event_factory(params),
        ResponseMessage::Data(params) => data_event_factory(params),
        ResponseMessage::Media(params) => media_event_factory(params),
    }
}

fn peer_service_factory(params: PeerServiceParams) -> (Parameter, Arc<dyn Service>) {
    use crate::di::*;

    match params {
        PeerServiceParams::Create { params } => {
            let module = PeerCreateServiceContainer::builder().build();
            let service: Arc<dyn Service> = module.resolve();
            (params, service)
        }
        PeerServiceParams::Status { params } => {
            let module = PeerStatusServiceContainer::builder().build();
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

fn data_service_factory(params: DataServiceParams) -> (Parameter, Arc<dyn Service>) {
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

fn media_service_factory(params: MediaServiceParams) -> (Parameter, Arc<dyn Service>) {
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
            // この値は使わないので何でも良い
            (Parameter(serde_json::Value::Null), service)
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
pub(crate) fn service_factory(params: ServiceParams) -> (Parameter, Arc<dyn Service>) {
    match params {
        ServiceParams::Peer(params) => peer_service_factory(params),
        ServiceParams::Data(params) => data_service_factory(params),
        ServiceParams::Media(params) => media_service_factory(params),
    }
}
