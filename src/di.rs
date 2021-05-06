use shaku::*;

use crate::application::usecase::peer::create::CreateService;
use crate::application::usecase::peer::delete::DeleteService;
use crate::application::usecase::peer::event::EventService;
use crate::domain::peer::value_object::PeerImpl;
use crate::domain::utility::ApplicationState;
use crate::infra::peer::{PeerApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};
use crate::infra::utility::ApplicationStateImpl;

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

module! {
    pub(crate) ApplicationStateContainer {
        components = [ApplicationStateImpl],
        providers = []
    }
}
