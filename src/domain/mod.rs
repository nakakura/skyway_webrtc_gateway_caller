// Domain層として機能を定義する
// 現時点では大きく2つの機能が存在する
// ・アプリケーションの起動状態を示すもの -> state module
//   (event loopからのexitの際に利用される)
// ・SkyWay WebRTC Gateway関連のもの -> webrtc module

/// アプリケーションが継続して実行されるべきかどうかを示す
pub(crate) mod state;
/// SkyWay WebRTC Gatewayを利用するための機能を定義する
pub(crate) mod webrtc;
