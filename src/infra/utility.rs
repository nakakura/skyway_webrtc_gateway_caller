use shaku::*;

use crate::domain::utility::ApplicationState;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
#[derive(Component)]
#[shaku(interface = ApplicationState)]
pub(crate) struct ApplicationStateImpl;

impl ApplicationState for ApplicationStateImpl {
    fn is_running(&self) -> bool {
        true
    }
}
