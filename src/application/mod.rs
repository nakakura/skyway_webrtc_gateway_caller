pub(crate) mod usecase;

use crate::application::usecase::value_object::ServiceParams;
use crate::application::usecase::value_object::{service_factory, ResponseMessage};

pub(crate) async fn run(params: ServiceParams) -> ResponseMessage {
    // 与えられたパラメータに応じて、各UseCaseをサービスとして生成し、同時にパラメータも生成する
    let (params, service) = service_factory(params);
    // UseCaseを実行する。
    // エラーが生じた場合も、エラーを生成するという正常動作と捉え、メッセージを返す
    crate::application::usecase::service::execute_service(service, params).await
}
