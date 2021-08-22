// このmoduleは、skyway-webrtc-gatewayのモジュールをそのまま再利用しており、
// ドメイン知識としての値のvalidationは、skyway-webrtc-gateway内部の機能として利用する
// このような再定義は、webrtcモジュール配下のvalue_objectのみに留め、
// その他のskyway-webrtc-gateway crateへの直接的な依存はinfra層に限定する
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error;

#[cfg(test)]
use mockall::automock;

/// skyway-webrtc-gateway-apiで定義されているオブジェクトのうち、/peer APIに関係するものを利用する。
/// これらは単なるパラメータであり、値自体のvalidationはskyway-webrtc-gateway-api crate内で行われる
pub use skyway_webrtc_gateway_api::peer::{
    PeerCallEvent, PeerCloseEvent, PeerConnectionEvent, PeerErrorEvent, PeerEventEnum, PeerId,
    PeerInfo, PeerOpenEvent, PeerStatusMessage, Token,
};

/// POST /peerで必要なパラメータ類
#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct CreatePeerParams {
    pub key: String,
    pub domain: String,
    pub peer_id: PeerId,
    pub turn: bool,
}

use crate::domain::webrtc::peer::repository::ReerRepositoryApi;

pub struct Peer {
    peer_info: PeerInfo,
    repository: Arc<dyn ReerRepositoryApi>,
}

#[cfg_attr(test, automock)]
impl Peer {
    pub async fn try_create(
        repository: Arc<dyn ReerRepositoryApi>,
        params: CreatePeerParams,
    ) -> Result<Self, error::Error> {
        let peer_info = repository.create(params).await?;
        loop {
            let result = repository.event(peer_info.clone()).await?;
            match result {
                PeerEventEnum::OPEN(event) => {
                    return Ok(Peer {
                        peer_info: event.params,
                        repository,
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

    pub async fn find(
        repository: Arc<dyn ReerRepositoryApi>,
        peer_info: PeerInfo,
    ) -> Result<(Self, PeerStatusMessage), error::Error> {
        let status = repository.status(&peer_info).await?;
        Ok((
            Peer {
                peer_info: peer_info,
                repository,
            },
            status,
        ))
    }

    pub async fn try_delete(&self) -> Result<PeerInfo, error::Error> {
        let _ = self.repository.delete(self.peer_info()).await?;
        Ok(self.peer_info.clone())
    }

    pub async fn try_event(&self) -> Result<PeerEventEnum, error::Error> {
        self.repository.event(self.peer_info.clone()).await
    }

    pub fn peer_info(&self) -> &PeerInfo {
        &self.peer_info
    }
}

#[cfg(test)]
mod test_peer_create {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::super::repository::MockReerRepositoryApi;
    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
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
        let mut api = MockReerRepositoryApi::default();
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

        // 生成に成功
        assert_eq!(peer.peer_info(), &expected);
    }

    #[tokio::test]
    async fn success_after_timeout() {
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
        let mut api = MockReerRepositoryApi::default();
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
                *mutex_value += 1;
                Ok(PeerEventEnum::TIMEOUT)
            } else {
                Ok(PeerEventEnum::OPEN(PeerOpenEvent { params: peer_info }))
            }
        });

        // 実行
        let peer = Peer::try_create(Arc::new(api), param).await.unwrap();

        // 生成に成功
        assert_eq!(peer.peer_info(), &expected);
    }

    #[tokio::test]
    async fn create_fail() {
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
        let mut api = MockReerRepositoryApi::default();
        api.expect_create()
            .return_once(move |_| Err(error::Error::create_local_error("peer create error")));

        // 実行
        let result = Peer::try_create(Arc::new(api), param).await;

        // createメソッドの実行失敗
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
        let mut api = MockReerRepositoryApi::default();
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

        // eventメソッドを実行した結果、異常なEVENTを受け取った
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
        let mut api = MockReerRepositoryApi::default();
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

        // eventメソッドの実行失敗
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "event fail".to_string());
        } else {
            assert!(false);
        }
    }
}

#[cfg(test)]
mod test_peer_find {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::super::repository::MockReerRepositoryApi;
    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 実行用パラメータの生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 正解値を生成
        let expected = PeerStatusMessage {
            peer_id: peer_info.peer_id().clone(),
            disconnected: false,
        };

        // 成功するパターンのMockを生成
        let mut api = MockReerRepositoryApi::default();
        let status = expected.clone();
        api.expect_status().return_once(move |_| Ok(status.clone()));

        // 実行
        let (peer, status) = Peer::find(Arc::new(api), peer_info.clone()).await.unwrap();

        // 作成に成功する
        assert_eq!(peer.peer_info(), &peer_info);
        assert_eq!(status, expected);
    }

    // 終了後であってもWebRTC Gatewayが過去のPeer Objectの状態を把握しており、statusを返してくる場合
    #[tokio::test]
    async fn closed_peer() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 実行用パラメータの生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 正解値を生成
        let expected = PeerStatusMessage {
            peer_id: peer_info.peer_id().clone(),
            disconnected: true,
        };

        // statusの取得には成功するパターンのMockを生成
        // Peerが解放済みなので、disconnectedはtrueで返す
        let mut api = MockReerRepositoryApi::default();
        let status = expected.clone();
        api.expect_status().return_once(move |_| Ok(status.clone()));

        // 実行
        let (_, status) = Peer::find(Arc::new(api), peer_info.clone()).await.unwrap();

        // disconnected trueのStatusを受け取る
        assert_eq!(status, expected);
    }

    #[tokio::test]
    async fn api_fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 実行用パラメータの生成
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // エラーを返すパターンのMockを生成
        let mut api = MockReerRepositoryApi::default();
        api.expect_status()
            .return_once(move |_| Err(error::Error::create_local_error("status api error")));

        // 実行
        let result = Peer::find(Arc::new(api), peer_info.clone()).await;

        // API Errorを返す
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(&e, "status api error");
        } else {
            assert!(false);
        }
    }
}

#[cfg(test)]
mod test_peer_delete {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::super::repository::MockReerRepositoryApi;
    use super::*;

    // Lock to prevent tests from running simultaneously
    static LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn success() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 正解値を生成
        let expected =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();

        // 成功するパターンのMockを生成
        let mut api = MockReerRepositoryApi::default();
        api.expect_delete().return_once(move |_| Ok(()));

        // パラメータのセットアップ
        let peer = Peer {
            peer_info: expected.clone(),
            repository: Arc::new(api),
        };

        // 実行
        let result = peer.try_delete().await.unwrap();

        // 作成に成功する
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn fail() {
        // mockのcontextが上書きされてしまわないよう、並列実行を避ける
        let _lock = LOCKER.lock();

        // 失敗するパターンのMockを生成
        let mut api = MockReerRepositoryApi::default();
        api.expect_delete()
            .return_once(move |_| Err(error::Error::create_local_error("delete method failed")));

        // パラメータのセットアップ
        let peer_info =
            PeerInfo::try_create("peer_id", "pt-9749250e-d157-4f80-9ee2-359ce8524308").unwrap();
        let peer = Peer {
            peer_info: peer_info,
            repository: Arc::new(api),
        };

        // 実行
        let result = peer.try_delete().await;

        // 作成に失敗する
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "delete method failed".to_string());
        } else {
            assert!(false);
        }
    }
}
