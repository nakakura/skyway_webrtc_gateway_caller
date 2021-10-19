use std::sync::Arc;

use crate::domain::webrtc::peer::entity::*;
use crate::domain::webrtc::peer::repository::PeerRepository;
use crate::domain::webrtc::peer::value_object::PeerInfo;
use crate::error;

#[cfg(test)]
use mockall::automock;

// automockでテストするためにmodでくるむ必要がある
#[cfg_attr(test, automock)]
pub mod create {
    use super::*;

    pub async fn execute(
        repository: Arc<dyn PeerRepository>,
        params: CreatePeerParams,
    ) -> Result<PeerInfo, error::Error> {
        let peer_info = repository.create(params).await?;
        loop {
            let result = repository.event(peer_info.clone()).await?;
            match result {
                PeerEventEnum::OPEN(event) => {
                    return Ok(event.params);
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
}

#[cfg(test)]
mod test_peer_create {
    use std::sync::Mutex;

    use super::super::repository::MockPeerRepository;
    use super::*;

    #[tokio::test]
    async fn success() {
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
        let mut api = MockPeerRepository::default();
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
        let peer_info = create::execute(Arc::new(api), param).await.unwrap();

        // 生成に成功
        assert_eq!(peer_info, expected);
    }

    #[tokio::test]
    async fn success_after_timeout() {
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
        let mut api = MockPeerRepository::default();
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
        let peer_info = create::execute(Arc::new(api), param).await.unwrap();

        // 生成に成功
        assert_eq!(peer_info, expected);
    }

    #[tokio::test]
    async fn create_fail() {
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
        let mut api = MockPeerRepository::default();
        api.expect_create()
            .return_once(move |_| Err(error::Error::create_local_error("peer create error")));

        // 実行
        let result = create::execute(Arc::new(api), param).await;

        // createメソッドの実行失敗
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "peer create error".to_string());
        } else {
            assert!(false);
        }
    }

    #[tokio::test]
    async fn create_success_but_wrong_event() {
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
        let mut api = MockPeerRepository::default();
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
        let result = create::execute(Arc::new(api), param).await;

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
        let mut api = MockPeerRepository::default();
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
        let result = create::execute(Arc::new(api), param).await;

        // eventメソッドの実行失敗
        if let Err(error::Error::LocalError(e)) = result {
            assert_eq!(e, "event fail".to_string());
        } else {
            assert!(false);
        }
    }
}
