use shaku::*;

use crate::domain::peer::value_object::PeerEvent;
use crate::infra::peer::{PeerControlApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};

module! {
    pub(crate) PeerRepositoryContainer {
        components = [PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerControlApiContainer {
        components = [PeerEvent, PeerControlApiImpl],
        providers = []
    }
}
