use usecase::factory::service_factory;

pub(crate) mod dto;
pub(crate) mod usecase;

use dto::request_message::ServiceParams;
use dto::response_message::ResponseMessage;

pub(crate) async fn run(params: ServiceParams) -> ResponseMessage {
    // 与えられたパラメータに応じて、各UseCaseをサービスとして生成し、同時にパラメータも生成する
    let (params, service) = service_factory(params);

    // UseCaseの実行
    let result = service.execute(params).await;
    // 結果をResponseMessageとして返す。
    // エラーが生じた場合も、エラーを生成するという正常動作と捉え、メッセージを返す
    match result {
        Ok(message) => message,
        Err(e) => ResponseMessage::Error(serde_json::to_string(&e).expect("create error failed")),
    }
}
