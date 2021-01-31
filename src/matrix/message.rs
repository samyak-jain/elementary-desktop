use std::{collections::HashSet, time::SystemTime};

use matrix_sdk::{
    events::{
        room::message::{MessageEventContent, Relation, TextMessageEventContent},
        AnyMessageEvent, AnyRoomEvent, AnyStateEvent, MessageEvent,
    },
    identifiers::EventId,
};

pub trait AnyMessageEventExt {
    fn image_url(&self) -> Option<String>;
}
impl AnyMessageEventExt for AnyMessageEvent {
    fn image_url(&self) -> Option<String> {
        match self {
            AnyMessageEvent::RoomMessage(MessageEvent {
                content: MessageEventContent::Image(ref image),
                ..
            }) => image.url.clone(),
            _ => None,
        }
    }
}

pub trait AnyRoomEventExt {
    /// Gets the event id of the underlying event
    fn event_id(&self) -> &EventId;
    /// Gets the ´origin_server_ts` member of the underlying event
    fn origin_server_ts(&self) -> SystemTime;
    /// Gets the mxc url in a message event if there is noe
    fn image_url(&self) -> Option<String>;
}

impl AnyRoomEventExt for AnyRoomEvent {
    fn event_id(&self) -> &EventId {
        match self {
            AnyRoomEvent::Message(e) => e.event_id(),
            AnyRoomEvent::State(e) => e.event_id(),
            AnyRoomEvent::RedactedMessage(e) => e.event_id(),
            AnyRoomEvent::RedactedState(e) => e.event_id(),
        }
    }
    fn origin_server_ts(&self) -> SystemTime {
        match self {
            AnyRoomEvent::Message(e) => e.origin_server_ts(),
            AnyRoomEvent::State(e) => e.origin_server_ts(),
            AnyRoomEvent::RedactedMessage(e) => e.origin_server_ts(),
            AnyRoomEvent::RedactedState(e) => e.origin_server_ts(),
        }
        .to_owned()
    }
    fn image_url(&self) -> Option<String> {
        match self {
            AnyRoomEvent::Message(message) => message.image_url(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MessageBuffer {
    /// The messages we have stored
    messages: Vec<AnyRoomEvent>,
    /// Set of event id's we have
    known_ids: HashSet<EventId>,
    /// Token for the start of the messages we have
    start: Option<String>,
    /// Token for the end of the messages we have
    end: Option<String>,
    /// Most recent activity in the room
    updated: std::time::SystemTime,
    /// Whether we're awaiting for backfill to be received
    loading: bool,
}

impl MessageBuffer {
    /// Sorts the messages by send time
    fn sort(&mut self) {
        self.messages
            .sort_unstable_by_key(|msg| msg.origin_server_ts())
    }
    /// Gets the send time of the most recently sent message
    fn update_time(&mut self) {
        self.updated = match self.messages.last() {
            Some(message) => message.origin_server_ts(),
            None => SystemTime::UNIX_EPOCH,
        };
    }
    fn remove(&mut self, id: &EventId) {
        self.messages.retain(|e| e.event_id() != id);
        self.known_ids.remove(&id);
    }
    /// Add a message to the buffer.
    pub fn push(&mut self, event: AnyRoomEvent) {
        self.known_ids.insert(event.event_id().clone());
        if let AnyRoomEvent::Message(AnyMessageEvent::RoomMessage(
            matrix_sdk::events::MessageEvent {
                content:
                    MessageEventContent::Text(TextMessageEventContent {
                        relates_to: Some(Relation::Replacement(ref replacement)),
                        ..
                    }),
                ..
            },
        )) = event
        {
            self.remove(&replacement.event_id);
        }
        if let AnyRoomEvent::Message(AnyMessageEvent::RoomRedaction(ref redaction)) = event {
            self.remove(&redaction.redacts);
        }
        self.messages.push(event);
        self.sort();
        self.update_time();
    }
    /// Adds several messages to the buffer
    pub fn append(&mut self, mut events: Vec<AnyRoomEvent>) {
        events.retain(|e| !self.known_ids.contains(e.event_id()));
        for event in events.iter() {
            // Handle replacement
            if let AnyRoomEvent::Message(AnyMessageEvent::RoomMessage(
                matrix_sdk::events::MessageEvent {
                    content:
                        MessageEventContent::Text(TextMessageEventContent {
                            relates_to: Some(Relation::Replacement(replacement)),
                            ..
                        }),
                    ..
                },
            )) = event
            {
                self.remove(&replacement.event_id);
            }
            if let AnyRoomEvent::Message(AnyMessageEvent::RoomRedaction(redaction)) = event {
                self.remove(&redaction.redacts);
            }
            self.known_ids.insert(event.event_id().clone());
        }
        self.messages.append(&mut events);
        self.sort();
        self.update_time();
    }
    /// Whather the message buffer has the room creation event
    pub fn has_beginning(&self) -> bool {
        self.messages
            .iter()
            .any(|e| matches!(e, AnyRoomEvent::State(AnyStateEvent::RoomCreate(_))))
    }
}

impl Default for MessageBuffer {
    fn default() -> Self {
        Self {
            messages: Default::default(),
            known_ids: Default::default(),
            start: None,
            end: None,
            updated: SystemTime::UNIX_EPOCH,
            loading: false,
        }
    }
}
