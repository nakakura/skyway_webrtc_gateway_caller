use shaku::*;

use crate::infra::peer::{PeerRepositoryApiImpl, PeerRepositoryImpl};

module! {
    pub(crate) PeerRepositoryContainer {
        components = [PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}
