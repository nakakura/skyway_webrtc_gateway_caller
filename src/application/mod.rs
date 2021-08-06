pub(crate) mod usecase;

use crate::application::usecase::value_object::ServiceParams;
use crate::application::usecase::value_object::{service_factory, ResponseMessage};

pub(crate) async fn run(params: ServiceParams) -> ResponseMessage {
    let (params, service) = service_factory(params);
    crate::application::usecase::service::execute_service(service, params).await
}
