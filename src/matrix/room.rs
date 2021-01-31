use matrix_sdk::identifiers::{RoomAliasId, UserId};

use super::message::MessageBuffer;

#[derive(Clone, Debug, Default)]
pub struct RoomEntry {
    /// Cached calculated name
    pub name: String,
    /// Room topic
    pub topic: String,
    /// Canonical alias
    pub alias: Option<RoomAliasId>,
    /// Defined display name
    pub display_name: Option<String>,
    /// mxc url for the rooms avatar
    pub avatar: Option<String>,
    /// Person we're in a direct message with
    pub direct: Option<UserId>,
    /// Cache of messages
    pub messages: MessageBuffer,
}

impl RoomEntry {
    pub async fn from_sdk(room: &matrix_sdk::JoinedRoom) -> Self {
        Self {
            direct: room.direct_target(),
            name: room.display_name().await.unwrap_or_default(),
            topic: room.topic().unwrap_or_default(),
            alias: room.canonical_alias(),
            avatar: room.avatar_url(),
            ..Default::default()
        }
    }
}
