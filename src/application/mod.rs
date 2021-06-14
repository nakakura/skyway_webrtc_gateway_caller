use std::collections::HashMap;

#[cfg(test)]
use mockall::automock;
#[cfg(test)]
use mockall_double::double;
use tokio::sync::{mpsc, oneshot};

use crate::application::usecase::service::ServiceParams;
use crate::domain::peer::value_object::Token;

// TODO: まだtestでしか使っていない
#[allow(dead_code)]
pub type PeerEventHash = HashMap<Token, mpsc::Sender<String>>;

pub(crate) mod usecase;

// TODO: まだtestでしか使っていない
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub(crate) enum EventEnum {
    System(Token),
    Json(ServiceParams),
}

// Unit TestではなくIntegration Testでテストする
// こっちは差し替える
#[cfg_attr(test, automock)]
pub(crate) mod service_creator {
    // 何故かwarningが出るのでマクロを入れる
    #[allow(unused_imports)]
    use crate::application::usecase::service::{ReturnMessage, Service, ServiceParams};

    pub(crate) async fn create(params_string: String) -> ReturnMessage {
        use shaku::HasComponent;

        use crate::di::*;

        match serde_json::from_str::<ServiceParams>(&params_string) {
            Ok(ServiceParams::PEER_CREATE { params }) => {
                let module = PeerCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                service.execute(params).await
            }
            Ok(ServiceParams::PEER_DELETE { params }) => {
                let module = PeerDeleteServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                service.execute(params).await
            }
            Err(e) => ReturnMessage::ERROR(crate::ErrorMessage {
                result: false,
                command: "UNKNOWN".into(),
                error_message: format!("{:?}", e),
            }),
        }
    }
}

// 設計変更に伴いこちらを使う
#[cfg_attr(test, automock)]
pub(crate) mod service_creator_refactor {
    // 何故かwarningが出るのでマクロを入れる
    #[allow(unused_imports)]
    use crate::application::usecase::service::{ReturnMessage, Service, ServiceParams};

    // TODO: まだtestでしか使っていない
    #[allow(dead_code)]
    pub(crate) async fn create(params: ServiceParams) -> ReturnMessage {
        use shaku::HasComponent;

        use crate::di::*;

        match params {
            ServiceParams::PEER_CREATE { params } => {
                let module = PeerCreateServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                service.execute(params).await
            }
            ServiceParams::PEER_DELETE { params } => {
                let module = PeerDeleteServiceContainer::builder().build();
                let service: &dyn Service = module.resolve_ref();
                service.execute(params).await
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

#[cfg_attr(test, automock)]
pub(crate) mod router {
    #[cfg(test)]
    use mockall_double::double;

    #[allow(unused_imports)]
    use crate::application::usecase::service::ReturnMessage;
    use crate::domain::peer::value_object::PeerInfo;

    #[allow(unused_imports)]
    use super::{event, mpsc, oneshot, EventEnum, PeerEventHash};
    // 何故かwarningが出るのでマクロを入れる
    #[allow(unused_imports)]
    #[cfg_attr(test, double)]
    use super::service_creator_refactor;

    // TODO: まだtestでしか使っていない
    #[allow(dead_code)]
    pub(crate) async fn run_event(
        mut hash: PeerEventHash,
        system_tx: mpsc::Sender<EventEnum>,
        event_observer_tx: mpsc::Sender<(PeerInfo, mpsc::Receiver<String>)>,
        message: EventEnum,
        response_tx: oneshot::Sender<String>,
    ) -> PeerEventHash {
        match message {
            EventEnum::System(key) => {
                hash.remove(&key);
            }
            EventEnum::Json(params) => {
                let result = service_creator_refactor::create(params).await;
                // to_string always success
                let _ = response_tx.send(serde_json::to_string(&result).unwrap());

                if let ReturnMessage::PEER_CREATE(params) = result {
                    let (tx, rx) = mpsc::channel::<String>(100);
                    let _ = event_observer_tx.send((params.params.clone(), rx)).await;
                    tokio::spawn(event::event(params.params, tx, system_tx.clone()));
                }
            }
        }

        hash
    }
}

#[cfg(test)]
mod test_run_event {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::application::router::run_event;
    use crate::application::usecase::ErrorMessage;
    use crate::{CreatePeerSuccessMessage, PeerInfo, ReturnMessage};

    #[cfg_attr(test, double)]
    use super::service_creator_refactor;
    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn delete_hash_item() {
        let mut hash = PeerEventHash::new();
        // insert 1st item to the hash
        let token_1 = Token::try_create("pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (tx_1, _rx_1) = mpsc::channel::<String>(10);
        hash.insert(token_1.clone(), tx_1);
        // insert 2nd item to the hash
        let token_2 = Token::try_create("pt-abc9250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let (tx_2, _rx_2) = mpsc::channel::<String>(10);
        hash.insert(token_2.clone(), tx_2.clone());

        // 戻り値を受け取るためのチンネルを作成
        let (response_tx, _response_rx) = oneshot::channel::<String>();
        // eventが停止したことを示すためのチャンネルを作成
        let (fuse_tx, mut fuse_rx) = mpsc::channel(10);
        // peer eventが停止したことを示すためのチャンネルを作成
        let (event_observer_tx, mut event_observer_rx) = mpsc::channel(10);
        // execute
        let result = run_event(
            hash,
            fuse_tx,
            event_observer_tx,
            EventEnum::System(token_1),
            response_tx,
        )
        .await;

        // 戻ってきたhashには、token_2: tx_2のペアが入っているはず
        assert_eq!(result.len(), 1);
        assert!(result.contains_key(&token_2));

        // fuse_txは発火しない
        let result = fuse_rx.recv().await;
        assert!(result.is_none());
        // event_observer_txも発火しない
        let result = event_observer_rx.recv().await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn execute_service_peer_create_success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // PEER_CREATEメッセージを返すmockを作成
        let ctx = service_creator_refactor::create_context();
        let message = ReturnMessage::PEER_CREATE(CreatePeerSuccessMessage {
            result: true,
            command: "PEER_CREATE".to_string(),
            params: PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308")
                .unwrap(),
        });
        let ret_message = message.clone();
        ctx.expect().return_const(ret_message);

        // hashは初期状態の設定で空
        let hash = PeerEventHash::new();

        // 戻り値を受け取るためのチャンネルを作成
        let (response_tx, response_rx) = oneshot::channel::<String>();
        // peer eventが停止したことを示すためのチャンネルを作成
        let (fuse_tx, mut fuse_rx) = mpsc::channel(10);
        // eventのレシーバを受けた渡すためのチャネルを作成
        let (event_observer_tx, mut event_observer_rx) = mpsc::channel(10);

        // execute
        let command = r#"{
            "command": "PEER_CREATE",
            "params": {
                "base_url": "http://localhost",
                "key": "api_key",
                "domain": "localhost",
                "peer_id": "peer_id",
                "turn": true
            }
        }"#;
        let params = serde_json::from_str::<ServiceParams>(command).unwrap();
        let result_return = run_event(
            hash,
            fuse_tx,
            event_observer_tx,
            EventEnum::Json(params),
            response_tx,
        )
        .await;
        let result_channel = response_rx.await;

        // 戻ってきたhashには、token_2: tx_2のペアが入っているはず
        assert_eq!(result_return.len(), 0);
        // channelからはPEER_CREATEメッセージのJSON文字列が入っているはず
        let expected = serde_json::to_string(&message).unwrap();
        assert_eq!(result_channel.unwrap(), expected);

        // fuse_txは発火しない
        let result = fuse_rx.recv().await;
        assert!(result.is_none());
        // event_observer_txも発火しない
        if let Some((peer_info, _)) = event_observer_rx.recv().await {
            assert_eq!(peer_info.peer_id().as_str(), "peer_id");
            assert_eq!(
                peer_info.token().as_str(),
                "pt-9749250e-d157-4f80-9ee2-359ce8524308"
            );
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn execute_service_peer_create_failed() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // ERRORメッセージを返すmockを作成
        let ctx = service_creator_refactor::create_context();
        let message = ReturnMessage::ERROR(ErrorMessage {
            result: false,
            command: "PEER_CREATE".to_string(),
            error_message: "error".to_string(),
        });
        let ret_message = message.clone();
        ctx.expect().return_const(ret_message);

        // hashは初期状態の設定で空
        let hash = PeerEventHash::new();

        // 戻り値を受け取るためのチャンネルを作成
        let (response_tx, response_rx) = oneshot::channel::<String>();
        // peer eventが停止したことを示すためのチャンネルを作成
        let (fuse_tx, mut fuse_rx) = mpsc::channel(10);
        // eventのレシーバを受けた渡すためのチャネルを作成
        let (event_observer_tx, mut event_observer_rx) = mpsc::channel(10);

        // execute
        let command = r#"{
            "command": "PEER_CREATE",
            "params": {
                "base_url": "http://localhost",
                "key": "api_key",
                "domain": "localhost",
                "peer_id": "peer_id",
                "turn": true
            }
        }"#;
        let params = serde_json::from_str::<ServiceParams>(command).unwrap();
        let result_return = run_event(
            hash,
            fuse_tx,
            event_observer_tx,
            EventEnum::Json(params),
            response_tx,
        )
        .await;
        let result_channel = response_rx.await;

        // 戻ってきたhashには、token_2: tx_2のペアが入っているはず
        assert_eq!(result_return.len(), 0);
        // channelからはERRORメッセージのJSON文字列が入っているはず
        let expected = serde_json::to_string(&message).unwrap();
        assert_eq!(result_channel.unwrap(), expected);

        // fuse_txは発火しない
        let result = fuse_rx.recv().await;
        assert!(result.is_none());
        // event_observer_txも発火しない
        let result = event_observer_rx.recv().await;
        assert!(result.is_none());
    }
}
