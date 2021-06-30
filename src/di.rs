use shaku::*;

use crate::application::usecase::data;
use crate::application::usecase::peer;
use crate::domain::peer::value_object::PeerImpl;
use crate::infra::data::DataApiImpl;
use crate::infra::peer::{PeerApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};
use crate::infra::utility::ApplicationStateAlwaysTrueImpl;

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
        components = [peer::event::EventService, PeerImpl, PeerApiImpl, ApplicationStateAlwaysTrueImpl],
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

module! {
    pub(crate) DataDeleteServiceContainer {
        components = [data::delete::DeleteService, DataApiImpl],
        providers = []
    }
}

module! {
    pub(crate) DataConnectServiceContainer {
        components = [data::connect::ConnectService, DataApiImpl],
        providers = []
    }
}

module! {
    pub(crate) DataDisconnectServiceContainer {
        components = [data::disconnect::DisconnectService, DataApiImpl],
        providers = []
    }
}

module! {
    pub(crate) DataRedirectServiceContainer {
        components = [data::redirect::RedirectService, DataApiImpl],
        providers = []
    }
}

module! {
    pub(crate) DataEventServiceContainer {
        components = [data::event::EventService, DataApiImpl, ApplicationStateAlwaysTrueImpl],
        providers = []
    }
}
