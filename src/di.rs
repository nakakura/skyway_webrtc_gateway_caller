use shaku::*;

use crate::application::usecase::data;
use crate::application::usecase::media;
use crate::application::usecase::peer;
use crate::infra::state::ApplicationStateAlwaysTrueImpl;
use crate::infra::webrtc::data::DataRepositoryImpl;
use crate::infra::webrtc::media::MediaRepositoryImpl;
use crate::infra::webrtc::peer::PeerRepositoryImpl;

//========== Peer Refactor Service ==========

module! {
    pub(crate) PeerCreateServiceContainer {
        components = [peer::create::CreateService, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerStatusServiceContainer {
        components = [peer::status::StatusService, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerDeleteServiceContainer {
        components = [peer::delete::DeleteService, PeerRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) PeerEventServiceContainer {
        components = [peer::event::EventService, PeerRepositoryImpl, ApplicationStateAlwaysTrueImpl],
        providers = []
    }
}

//========== Data Service ==========
module! {
    pub(crate) DataCreateServiceContainer {
        components = [data::create::CreateService, DataRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) DataDeleteServiceContainer {
        components = [data::delete::DeleteService, DataRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) DataConnectServiceContainer {
        components = [data::connect::ConnectService, DataRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) DataDisconnectServiceContainer {
        components = [data::disconnect::DisconnectService, DataRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) DataRedirectServiceContainer {
        components = [data::redirect::RedirectService, DataRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) DataEventServiceContainer {
        components = [data::event::EventService, DataRepositoryImpl, ApplicationStateAlwaysTrueImpl],
        providers = []
    }
}

module! {
    pub(crate) DataStatusServiceContainer {
        components = [data::status::StatusService, DataRepositoryImpl],
        providers = []
    }
}

//========== Media Service ==========
module! {
    pub(crate) MediaContentCreateServiceContainer {
        components = [media::create_media::CreateMediaService, MediaRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaContentDeleteServiceContainer {
        components = [media::delete_media::DeleteMediaService, MediaRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaRtcpCreateServiceContainer {
        components = [media::create_rtcp::CreateRtcpService, MediaRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaRtcpDeleteServiceContainer {
        components = [media::delete_rtcp::DeleteRtcpService, MediaRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaCallServiceContainer {
        components = [media::call::CallService, MediaRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaAnswerServiceContainer {
        components = [media::answer::AnswerService, MediaRepositoryImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaEventServiceContainer {
        components = [media::event::EventService, MediaRepositoryImpl, ApplicationStateAlwaysTrueImpl],
        providers = []
    }
}

module! {
    pub(crate) MediaStatusServiceContainer {
        components = [media::status::StatusService, MediaRepositoryImpl],
        providers = []
    }
}
