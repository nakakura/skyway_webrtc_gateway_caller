// Domain層で定義されている機能を実装する
// 現状このレイヤで実装されている機能は以下の2つである
// ・アプリケーションが実行中であるかどうかを提示するStruct
// ・SkyWay WebRTC GatewayのAPIを叩くためのStruct

// 前者は、event loop内でのexit判定に利用される
// stateモジュールとして実装される
//
// 後者はskyway-webrtc-gateway-api crateの機能をDomain層の定義と合わせるための薄いラッパーである
// webrtcモジュールとして実装される

pub(crate) mod state;
pub(crate) mod webrtc;
