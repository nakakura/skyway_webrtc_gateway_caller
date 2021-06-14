use shaku::*;

use crate::application::usecase::data;
use crate::application::usecase::peer;
use crate::domain::peer::value_object::PeerImpl;
use crate::infra::data::DataApiImpl;
use crate::infra::peer::{PeerApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};
use crate::infra::utility::ApplicationStateImpl;

//========== Peer Service ==========
module! {
    pub(crate) PeerCreateServiceContainer {
        components = [peer::create::CreateService, PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerDeleteServiceContainer {
        components = [peer::delete::DeleteService, PeerRepositoryApiImpl, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerEventServiceContainer {
        components = [peer::event::EventService, PeerImpl, PeerApiImpl],
        providers = []
    }
}

//========== Data Service ==========
module! {
    pub(crate) DataCreateServiceContainer {
        components = [data::create::CreateService, DataApiImpl],
        providers = []
    }
}

//========== Util ==========
module! {
    pub(crate) ApplicationStateContainer {
        components = [ApplicationStateImpl],
        providers = []
    }
}
