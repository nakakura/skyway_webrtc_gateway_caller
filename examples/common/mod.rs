pub mod data;
pub mod peer;
pub mod terminal;

use rust_module::prelude::*;
use tokio::sync::oneshot;

pub type ControlMessage = (oneshot::Sender<ResponseMessage>, ServiceParams);
