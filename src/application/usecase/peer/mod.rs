pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod event;
pub(crate) mod status;

// Lock to prevent tests from running simultaneously
#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
use once_cell::sync::Lazy;

#[cfg(test)]
pub static PEER_FIND_MOCK_LOCKER: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
