use async_trait::async_trait;
use shaku::Interface;
use tokio::sync::mpsc::Sender;

use crate::application::dto::request_message::Parameter;
use crate::application::dto::response_message::ResponseMessage;
use crate::error;

#[cfg(test)]
use mockall::automock;

// 副作用のない単発のサービス
// WebRTC Gatewayを叩いて結果を返す
// create系のように、createのapiを叩いたあとopenイベントを確認するためevent apiを叩くものもあるが、
// return以外の結果の外部出力やステータスを持たない
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait Service: Interface {
    async fn execute(&self, params: Parameter) -> Result<ResponseMessage, error::Error>;
}

// WebRTC Gatewayのイベントを監視する
// Errorの発生もしくはCLOSEイベントの発火まで監視し続ける
// 終了理由をreturnする
// 個別の取得したイベントについては、TIMEOUTを除きexecuteメソッドで受け取ったSenderで通知する
#[cfg_attr(test, automock)]
#[async_trait]
pub(crate) trait EventListener: Interface {
    async fn execute(
        &self,
        event_tx: Sender<ResponseMessage>,
        params: Parameter,
    ) -> ResponseMessage;
}
