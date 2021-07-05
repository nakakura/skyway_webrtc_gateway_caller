pub(crate) mod data;
pub(crate) mod peer;
pub(crate) mod terminal;

use rust_module::prelude::*;
use tokio::sync::oneshot;

pub type ControlMessage = (oneshot::Sender<ResponseMessage>, ServiceParams);
