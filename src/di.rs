use shaku::*;

use crate::domain::peer::value_object::PeerImpl;
use crate::infra::peer::{PeerApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};
use crate::usecase::peer::create::CreateService;
use crate::usecase::peer::delete::DeleteService;
use crate::usecase::peer::event::EventService;

module! {
    pub(crate) PeerCreateServiceContainer {
        components = [CreateService, PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerDeleteServiceContainer {
        components = [DeleteService, PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerEventServiceContainer {
        components = [EventService, PeerImpl, PeerApiImpl],
        providers = []
    }
}
