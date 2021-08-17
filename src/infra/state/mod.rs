use shaku::*;

use crate::domain::state::ApplicationState;

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
// 常にtrueを返すStateの実装。これはテストを目的とした簡易的なものである
#[derive(Component)]
#[shaku(interface = ApplicationState)]
pub(crate) struct ApplicationStateAlwaysTrueImpl;

impl ApplicationState for ApplicationStateAlwaysTrueImpl {
    fn is_running(&self) -> bool {
        true
    }
}

// Serviceの具象Struct
// DIコンテナからのみオブジェクトを生成できる
// 常にfalseを返すStateの実装。これはテストを目的とした簡易的なものである
pub(crate) struct ApplicationStateAlwaysFalseImpl;

impl ApplicationState for ApplicationStateAlwaysFalseImpl {
    fn is_running(&self) -> bool {
        false
    }
}
