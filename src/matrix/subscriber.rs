use async_stream::stream;
use std::time::Duration;

use matrix_sdk::{
    events::{AnyRoomEvent, AnySyncRoomEvent, AnyToDeviceEvent},
    LoopCtrl, SyncSettings,
};

pub struct MatrixSync {
    client: matrix_sdk::Client,
    join: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub enum MatrixEvents {
    Room(AnyRoomEvent),
    ToDevice(AnyToDeviceEvent),
}

impl<H, I> iced_native::subscription::Recipe<H, I> for MatrixSync
where
    H: std::hash::Hasher,
{
    type Output = MatrixEvents;
    fn hash(&self, state: &mut H) {
        use std::hash::Hash;
        std::any::TypeId::of::<Self>().hash(state);
        //self.id.hash(state);
    }
    fn stream(
        mut self: Box<Self>,
        _input: iced_futures::BoxStream<I>,
    ) -> iced_futures::BoxStream<Self::Output> {
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let client = self.client.clone();
        let join = tokio::task::spawn(async move {
            client
                .sync_with_callback(
                    SyncSettings::new()
                        .token(client.sync_token().await.unwrap())
                        .timeout(Duration::from_secs(30)),
                    |response| async {
                        //sender.send(Event::Token(response.next_batch)).ok();
                        for (id, room) in response.rooms.join {
                            for event in room.state.events {
                                let id = id.clone();
                                sender
                                    .send(Self::Output::Room(AnyRoomEvent::State(
                                        event.into_full_event(id),
                                    )))
                                    .ok();
                            }
                            for event in room.timeline.events {
                                let id = id.clone();
                                let event = match event {
                                    AnySyncRoomEvent::Message(e) => {
                                        AnyRoomEvent::Message(e.into_full_event(id))
                                    }
                                    AnySyncRoomEvent::State(e) => {
                                        AnyRoomEvent::State(e.into_full_event(id))
                                    }
                                    AnySyncRoomEvent::RedactedMessage(e) => {
                                        AnyRoomEvent::RedactedMessage(e.into_full_event(id))
                                    }
                                    AnySyncRoomEvent::RedactedState(e) => {
                                        AnyRoomEvent::RedactedState(e.into_full_event(id))
                                    }
                                };
                                sender.send(Self::Output::Room(event)).ok();
                            }
                        }
                        for event in response.to_device.events {
                            sender.send(Self::Output::ToDevice(event)).ok();
                        }
                        LoopCtrl::Continue
                    },
                )
                .await;
        });
        self.join = Some(join);
        let stream = stream! {
            while let Some(item) = receiver.recv().await {
                yield item;
            }
        };
        Box::pin(stream)
    }
}

impl MatrixSync {
    pub fn subscription(client: matrix_sdk::Client) -> iced::Subscription<MatrixEvents> {
        iced::Subscription::from_recipe(MatrixSync { client, join: None })
    }
}
