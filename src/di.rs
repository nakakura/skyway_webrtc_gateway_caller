use shaku::*;

use crate::domain::peer::value_object::PeerImpl;
use crate::infra::peer::{PeerApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};

module! {
    pub(crate) PeerRepositoryContainer {
        components = [PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerApiContainer {
        components = [PeerImpl, PeerApiImpl],
        providers = []
    }
}
