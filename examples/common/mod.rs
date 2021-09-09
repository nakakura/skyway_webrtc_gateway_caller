use tokio::sync::oneshot;

use rust_module::prelude::*;

pub mod data;
pub mod media;
pub mod peer;
pub mod terminal;

pub type ControlMessage = (oneshot::Sender<ResponseMessage>, ServiceParams);
