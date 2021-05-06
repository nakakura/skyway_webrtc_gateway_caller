use std::sync::Arc;

use shaku::Interface;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub(crate) trait ApplicationState: Interface {
    fn is_running(&self) -> bool;
}
