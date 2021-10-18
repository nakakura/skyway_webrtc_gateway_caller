use tokio::sync::oneshot;

pub mod data;
pub mod media;
pub mod peer;
pub mod terminal;

pub type ControlMessage = (oneshot::Sender<String>, String);
