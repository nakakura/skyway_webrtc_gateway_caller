use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;
use shaku::HasComponent;

use crate::application::usecase::service::EventListener;
use crate::application::usecase::value_object::{
    DataResponseMessageBodyEnum, MediaResponseMessageBodyEnum, PeerResponseMessageBodyEnum,
    ResponseMessageBodyEnum,
};

fn value<V: Serialize, T: HasComponent<dyn EventListener>>(
    param: V,
    component: T,
) -> (Value, Arc<dyn EventListener>) {
    // paramsはserializeをimplementしているので、エラーが出ることはなく、unwrapで問題ない
    let value = serde_json::to_value(&param).unwrap();
    (value, component.resolve())
}

fn peer_event_factory(
    params: PeerResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        PeerResponseMessageBodyEnum::Create(params) => {
            let component = PeerEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

fn data_event_factory(
    params: DataResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        DataResponseMessageBodyEnum::Connect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        DataResponseMessageBodyEnum::Redirect(params) => {
            let component = DataEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

fn media_event_factory(
    params: MediaResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    use crate::di::*;

    match params {
        MediaResponseMessageBodyEnum::Call(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        MediaResponseMessageBodyEnum::Answer(params) => {
            let component = MediaEventServiceContainer::builder().build();
            Some(value(params, component))
        }
        _ => None,
    }
}

// FIXME: no test
pub(crate) fn event_factory(
    message: ResponseMessageBodyEnum,
) -> Option<(Value, std::sync::Arc<dyn EventListener>)> {
    match message {
        ResponseMessageBodyEnum::Peer(params) => peer_event_factory(params),
        ResponseMessageBodyEnum::Data(params) => data_event_factory(params),
        ResponseMessageBodyEnum::Media(params) => media_event_factory(params),
    }
}
