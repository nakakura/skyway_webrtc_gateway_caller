// skyway-webrtc-gateway crateを利用するためのラッパーを実装するモジュール
// このモジュールは、SkyWay WebRTC GatewayのAPI区分に従ってサブモジュール化される

/// /date APIに対応するモジュール
pub(crate) mod data;
/// /media APIに対応するモジュール
pub(crate) mod media;
/// /peer APIに対応するモジュール
pub(crate) mod peer;
pub(crate) mod peer_refactor;
