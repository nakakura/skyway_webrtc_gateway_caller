use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use shaku::Interface;
use tokio::sync::mpsc::Sender;

use crate::application::usecase::value_object::ResponseMessage;
use crate::error;

#[cfg(test)]
use mockall::automock;

pub(crate) async fn execute_service(service: Arc<dyn Service>, params: Value) -> ResponseMessage {
    let result = service.execute(params).await;

    match result {
        Ok(message) => message,
        Err(e) => ResponseMessage::Error(serde_json::to_string(&e).expect("create error failed")),
    }
}

// 副作用のない単発のサービス
// WebRTC Gatewayを叩いて結果を返す
// create系のように、createのapiを叩いたあとopenイベントを確認するためevent apiを叩くものもあるが、
// return以外の結果の外部出力やステータスを持たない
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait Service: Interface {
    async fn execute(&self, params: Value) -> Result<ResponseMessage, error::Error>;
}

// WebRTC Gatewayのイベントを監視する
// Errorの発生もしくはCLOSEイベントの発火まで監視し続ける
// 終了理由をreturnする
// 個別の取得したイベントについては、TIMEOUTを除きexecuteメソッドで受け取ったSenderで通知する
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait EventListener: Interface {
    async fn execute(&self, event_tx: Sender<ResponseMessage>, params: Value) -> ResponseMessage;
}
