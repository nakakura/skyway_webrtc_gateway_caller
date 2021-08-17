use shaku::Interface;

#[cfg(test)]
use mockall::automock;

/// SkyWay WebRTC Gatewayのevent APIは、CLOSE eventを発行後、いかなるイベントも発火させない
/// そのためCLOSE event受領時にevent監視ループは終了するように実装されるべきである。
/// 一方で、まだeventが発火しうる状態であっても、crateを利用するアプリケーションが終了する場合など、
/// イベント監視ループを終了させたい場合がある。
/// そのようなケースであることを、このtraitを実装したオブジェクトで示すことができる。
/// is_runningメソッドでfalseを返すことで、イベント監視ループを抜けることが保証される。
#[cfg_attr(test, automock)]
pub(crate) trait ApplicationState: Interface {
    /// 継続してeventを監視し続けたい場合はtrue, 終了したい場合はfalseを返す
    fn is_running(&self) -> bool;
}
