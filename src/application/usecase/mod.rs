pub(crate) mod data;
pub(crate) mod peer;
pub(crate) mod service;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Ord, Hash)]
pub struct ErrorMessage {
    pub result: bool, // should be false
    pub command: String,
    pub error_message: String,
}
