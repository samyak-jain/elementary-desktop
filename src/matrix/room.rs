use std::collections::BTreeMap;

use futures::executor::block_on;
use matrix_sdk::{
    events::{room::message::MessageEventContent, MessageEvent},
    identifiers::{RoomAliasId, RoomId, UserId},
    Client, JoinedRoom,
};

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

    pub message_list: Vec<MessageEvent<MessageEventContent>>,
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

pub fn partition_rooms<'a>(
    rooms: &'a BTreeMap<RoomId, RoomEntry>,
    client: &Client,
) -> (
    Vec<(&'a RoomId, &'a RoomEntry)>,
    Vec<(&'a RoomId, &'a RoomEntry)>,
) {
    rooms
        .iter()
        // Hide if we're in the room the tombstone points to
        .filter(|(id, _)| {
            !client
                .get_joined_room(id)
                .and_then(|j| j.tombstone())
                .map(|t| rooms.contains_key(&t.replacement_room))
                .unwrap_or(false)
        })
        .partition(|(_, room)| room.direct.is_some())
}

pub fn get_sender_details(user: UserId, room: JoinedRoom) -> (String, Option<String>) {
    match block_on(async { room.get_member(&user).await.unwrap() }) {
        Some(member) => {
            return (
                String::from(member.name()),
                member.avatar_url().map(|url| String::from(url)),
            );
        }
        None => return (String::from("Unknown Sender"), None),
    };
}
