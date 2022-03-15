///　Channel handles only JSON format String.
/// response_parser provides objects in case you want to parse JSON String as a Rust object.
pub mod response_parser {
    pub use crate::application::dto::response_message::*;
}

/// Provide objects referenced by some categories
pub mod common {
    pub use crate::domain::webrtc::common::value_object::*;
}

/// Provide objects related to Data-based APIs
pub mod data {
    pub use crate::domain::webrtc::data::entity::*;
    pub use crate::domain::webrtc::data::value_object::*;
}

/// Provide objects related to Data-based APIs
pub mod media {
    pub use crate::domain::webrtc::media::entity::*;
    pub use crate::domain::webrtc::media::value_object::*;
}

/// Provide objects related to Data-based APIs
pub mod peer {
    pub use crate::domain::webrtc::peer::entity::*;
    pub use crate::domain::webrtc::peer::value_object::*;
}
