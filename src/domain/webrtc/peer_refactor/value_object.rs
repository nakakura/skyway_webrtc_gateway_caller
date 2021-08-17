// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
/// これらは単なるパラメータであり、値自体のvalidationはskyway-webrtc-gateway-api crate内で行われる
pub use skyway_webrtc_gateway_api::peer::{
    PeerCallEvent, PeerCloseEvent, PeerConnectionEvent, PeerErrorEvent, PeerEventEnum, PeerId,
    PeerInfo, PeerOpenEvent, Token,
};

/// POST /peerで必要なパラメータ類
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}

use crate::domain::webrtc::peer_refactor::repository::PeerRepositoryApi;

pub struct Peer {
    peer_info: PeerInfo,
}

impl Peer {
    pub async fn try_create(
        repository: Arc<dyn PeerRepositoryApi>,
        params: CreatePeerParams,
    ) -> Result<Self, error::Error> {
        let peer_info = repository.create(params).await?;
        loop {
            let result = repository.event(peer_info.clone()).await?;
            match result {
                PeerEventEnum::OPEN(event) => {
                    return Ok(Peer {
                        peer_info: event.params,
                    })
                }
                PeerEventEnum::TIMEOUT => {
                    continue;
                }
                _ => {
                    return Err(error::Error::create_local_error(
                        "not receiving OPEN event in the PeerCreate flow",
                    ));
                }
            }
        }
    }

    pub fn peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

#[cfg(test)]
mod test_peer {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::super::repository::MockPeerRepositoryApi;
    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn create_success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 実行パラメータを生成
        let param = CreatePeerParams {
            key: "API_KEY".to_string(),
            domain: "localhost".to_string(),
            peer_id: expected.peer_id(),
            turn: false,
        };

        // 成功するパターンのMockを生成
        let mut api = MockPeerRepositoryApi::default();
        api.expect_create()
            .return_once(move |params: CreatePeerParams| {
                PeerInfo::try_create(
                    params.peer_id.as_str(),
                    "pt-9749250e-d157-4f80-9ee2-359ce8524308",
                )
            });
        api.expect_event().return_once(move |peer_info: PeerInfo| {
            Ok(PeerEventEnum::OPEN(PeerOpenEvent { params: peer_info }))
        });

        // 実行
        let peer = Peer::try_create(Arc::new(api), param).await.unwrap();

        // 評価
        assert_eq!(peer.peer_info, expected);
    }

    #[tokio::test]
    async fn create_success_after_timeout() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 実行パラメータを生成
        let param = CreatePeerParams {
            key: "API_KEY".to_string(),
            domain: "localhost".to_string(),
            peer_id: expected.peer_id(),
            turn: false,
        };

        // Timeoutが帰ってきた後に成功するパターンのMockを生成
        let mut api = MockPeerRepositoryApi::default();
        api.expect_create()
            .return_once(move |params: CreatePeerParams| {
                PeerInfo::try_create(
                    params.peer_id.as_str(),
                    "pt-9749250e-d157-4f80-9ee2-359ce8524308",
                )
            });
        let counter = Mutex::new(0u8);
        api.expect_event().returning(move |peer_info: PeerInfo| {
            let mut mutex_value = counter.lock().unwrap();
            if *mutex_value == 0 {
                println!("hoge");
                *mutex_value += 1;
                Ok(PeerEventEnum::TIMEOUT)
            } else {
                Ok(PeerEventEnum::OPEN(PeerOpenEvent { params: peer_info }))
            }
        });

        // 実行
        let peer = Peer::try_create(Arc::new(api), param).await.unwrap();

        // 評価
        assert_eq!(peer.peer_info, expected);
    }

    #[tokio::test]
    async fn create_err() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 実行パラメータを生成
        let param = CreatePeerParams {
            key: "API_KEY".to_string(),
            domain: "localhost".to_string(),
            peer_id: expected.peer_id(),
            turn: false,
        };

        // createに失敗するパターンのMockを生成
        let mut api = MockPeerRepositoryApi::default();
        api.expect_create()
            .return_once(move |_| Err(error::Error::create_local_error("peer create error")));

        // 実行
        let result = Peer::try_create(Arc::new(api), param).await;

        // 評価
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "peer create error".to_string());
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn create_success_but_wrong_event() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 実行パラメータを生成
        let param = CreatePeerParams {
            key: "API_KEY".to_string(),
            domain: "localhost".to_string(),
            peer_id: expected.peer_id(),
            turn: false,
        };

        // 間違ったイベントが帰ってくるパターンのMockを生成
        let mut api = MockPeerRepositoryApi::default();
        api.expect_create()
            .return_once(move |params: CreatePeerParams| {
                PeerInfo::try_create(
                    params.peer_id.as_str(),
                    "pt-9749250e-d157-4f80-9ee2-359ce8524308",
                )
            });
        api.expect_event().return_once(move |peer_info: PeerInfo| {
            Ok(PeerEventEnum::CLOSE(PeerCloseEvent { params: peer_info }))
        });

        // 実行
        let result = Peer::try_create(Arc::new(api), param).await;

        // 評価
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(
                e,
                "not receiving OPEN event in the PeerCreate flow".to_string()
            );
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn create_success_but_event_fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 実行パラメータを生成
        let param = CreatePeerParams {
            key: "API_KEY".to_string(),
            domain: "localhost".to_string(),
            peer_id: expected.peer_id(),
            turn: false,
        };

        // 間違ったイベントが帰ってくるパターンのMockを生成
        let mut api = MockPeerRepositoryApi::default();
        api.expect_create()
            .return_once(move |params: CreatePeerParams| {
                PeerInfo::try_create(
                    params.peer_id.as_str(),
                    "pt-9749250e-d157-4f80-9ee2-359ce8524308",
                )
            });
        api.expect_event()
            .return_once(move |_| Err(error::Error::create_local_error("event fail")));

        // 実行
        let result = Peer::try_create(Arc::new(api), param).await;

        // 評価
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "event fail".to_string());
        } else {
            assert!(false);
        }
    }
}
