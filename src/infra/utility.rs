use shaku::*;

use crate::domain::utility::ApplicationState;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = ApplicationState)]
pub(crate) struct ApplicationStateAlwaysTrueImpl;

impl ApplicationState for ApplicationStateAlwaysTrueImpl {
    fn is_running(&self) -> bool {
        true
    }
}

pub(crate) struct ApplicationStateAlwaysFalseImpl;

impl ApplicationState for ApplicationStateAlwaysFalseImpl {
    fn is_running(&self) -> bool {
        false
    }
}
