use shaku::*;

use crate::application::usecase::data;
use crate::application::usecase::media;
use crate::application::usecase::peer;
use crate::domain::webrtc::peer::value_object::PeerImpl;
use crate::infra::state::ApplicationStateAlwaysTrueImpl;
use crate::infra::webrtc::data::DataApiImpl;
use crate::infra::webrtc::media::MediaApiImpl;
use crate::infra::webrtc::peer::{PeerApiImpl, PeerRepositoryApiImpl, PeerRepositoryImpl};

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

//========== Media Service ==========
module! {
    pub(crate) MediaContentCreateServiceContainer {
        components = [media::create_media::CreateMediaService, MediaApiImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaContentDeleteServiceContainer {
        components = [media::delete_media::DeleteMediaService, MediaApiImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaRtcpCreateServiceContainer {
        components = [media::create_rtcp::CreateRtcpService, MediaApiImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaRtcpDeleteServiceContainer {
        components = [media::delete_rtcp::DeleteRtcpService, MediaApiImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaCallServiceContainer {
        components = [media::call::CallService, MediaApiImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaAnswerServiceContainer {
        components = [media::answer::AnswerService, MediaApiImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaEventServiceContainer {
        components = [media::event::EventService, MediaApiImpl, ApplicationStateAlwaysTrueImpl],
        providers = []
    }
}
