use futures::executor::block_on;
use iced::{
    container, Align, Button, Column, Command, Container, Element, Image, Length, Row, Rule,
    Scrollable, Text, TextInput,
};
use matrix_sdk::{
    events::{
        room::{member::MembershipState, message::MessageEventContent},
        AnyMessageEvent, AnyMessageEventContent, AnyRoomEvent, AnyStateEvent,
    },
    identifiers::{RoomId, UserId},
};

use crate::{
    matrix::{
        message::{parse_mxc, AnyMessageEventExt, AnyRoomEventExt},
        room::{get_sender_details, partition_rooms, RoomEntry},
        subscriber::MatrixEvents,
    },
    theme::{dark, style::Theme},
};

use matrix_sdk::api::r0::media::get_content::Request as ImageRequest;
use matrix_sdk::api::r0::message::get_message_events::Request as MessageRequest;

use super::{HomePage, Messages};

impl HomePage {
    pub fn update(&mut self, message: Messages) -> iced::Command<Messages> {
        match message {
            Messages::Sync(event) => match event {
                MatrixEvents::Room(room_event) => match room_event {
                    matrix_sdk::events::AnyRoomEvent::Message(message_event) => {
                        //room.messages.push(AnyRoomEvent::Message(message_event.clone()));

                        let mut commands = Vec::new();
                        let send_message_event = message_event.clone();
                        commands
                            .push(async move { Messages::RoomMessage(send_message_event) }.into());

                        if self.selected.as_ref() == Some(message_event.room_id()) {
                            let client = self.client.clone();
                            let marker_cmd = async move {
                                let result = client
                                    .read_marker(
                                        message_event.room_id(),
                                        message_event.event_id(),
                                        Some(message_event.event_id()),
                                    )
                                    .await
                                    .err();

                                match result {
                                    Some(err) => Messages::LoginFailed(err.to_string()),
                                    None => Messages::LoginFailed(String::from("test")),
                                }
                            };

                            commands.push(marker_cmd.into());
                        };

                        return Command::batch(commands);
                    }
                    matrix_sdk::events::AnyRoomEvent::State(event) => {
                        match event {
                            AnyStateEvent::RoomCanonicalAlias(ref alias) => {
                                let room = self.rooms.entry(alias.room_id.clone()).or_default();
                                room.alias = alias.content.alias.clone();
                                //room.messages.push(AnyRoomEvent::State(event));
                            }
                            AnyStateEvent::RoomName(ref name) => {
                                let id = name.room_id.clone();
                                let room = self.rooms.entry(id.clone()).or_default();
                                room.display_name = name.content.name().map(String::from);
                                //room.messages.push(AnyRoomEvent::State(event));
                                let client = self.client.clone();
                                return async move {
                                    let joined = client.get_joined_room(&id).unwrap();
                                    Messages::RoomName(id, joined.display_name().await.unwrap())
                                }
                                .into();
                            }
                            AnyStateEvent::RoomTopic(ref topic) => {
                                let room = self.rooms.entry(topic.room_id.clone()).or_default();
                                room.topic = topic.content.topic.clone();
                                //room.messages.push(AnyRoomEvent::State(event));
                            }
                            AnyStateEvent::RoomAvatar(ref avatar) => {
                                let room = self.rooms.entry(avatar.room_id.clone()).or_default();
                                //room.messages.push(AnyRoomEvent::State(event));
                                if let Some(url) = room.avatar.clone() {
                                    room.avatar = Some(url.clone());
                                    return async { Messages::FetchImage(url) }.into();
                                }
                            }
                            AnyStateEvent::RoomCreate(ref create) => {
                                // Add room to the entry list
                                let joined = self.client.get_joined_room(&create.room_id).unwrap();
                                let id = create.room_id.clone();
                                return async move {
                                    let entry = RoomEntry::from_sdk(&joined).await;
                                    //entry.messages.push(AnyRoomEvent::State(event));
                                    Messages::ResetRoom(id, entry)
                                }
                                .into();
                            }
                            AnyStateEvent::RoomMember(ref member) => {
                                //let room = self.rooms.entry(member.room_id.clone()).or_default();
                                let client = self.client.clone();
                                // If we left a room, remove it from the RoomEntry list
                                if member.state_key == self.session.user_id {
                                    match member.content.membership {
                                        MembershipState::Join => {
                                            let id = member.room_id.clone();
                                            return async move {
                                                let joined = client.get_joined_room(&id).unwrap();
                                                let entry = RoomEntry::from_sdk(&joined).await;
                                                Messages::ResetRoom(id, entry)
                                            }
                                            .into();
                                        }
                                        MembershipState::Leave => {
                                            // Deselect room if we're leaving selected room
                                            if self.selected.as_ref() == Some(&member.room_id) {
                                                self.selected = None;
                                            }
                                            self.rooms.remove(&member.room_id);
                                            return Command::none();
                                        }
                                        _ => (),
                                    }
                                }
                                //room.messages.push(AnyRoomEvent::State(event));
                            }
                            ref any => {
                                //    Ensure room exists
                                let room = self.rooms.entry(any.room_id().clone()).or_default();
                                room.messages.push(AnyRoomEvent::State(event));
                            }
                        }
                    }
                    matrix_sdk::events::AnyRoomEvent::RedactedMessage(_) => {}
                    matrix_sdk::events::AnyRoomEvent::RedactedState(_) => {}
                },
                MatrixEvents::ToDevice(_) => {}
            },
            Messages::RoomMessage(message_event) => {
                let room = self
                    .rooms
                    .entry(message_event.room_id().clone())
                    .or_default();

                match message_event.clone() {
                    AnyMessageEvent::Reaction(_) => {}
                    AnyMessageEvent::RoomEncrypted(_) => {}
                    AnyMessageEvent::RoomMessage(message) => {
                        println!("Message: {:#?}", message);

                        room.message_list.push(message.clone());
                        println!("Message List: {:#?}", room.message_list);

                        if let MessageEventContent::Image(image_message_content) = message.content {
                            if let Some(image_url) = image_message_content.url {
                                return async move { Messages::FetchImage(image_url) }.into();
                            }
                        }
                    }
                    AnyMessageEvent::RoomMessageFeedback(_) => {}
                    AnyMessageEvent::RoomRedaction(_) => {}
                    AnyMessageEvent::Sticker(_) => {}
                    _ => {}
                };
            }
            Messages::ResetRoom(id, room) => {
                self.rooms.insert(id.clone(), room);
                return async move { Messages::BackFill(id) }.into();
            }
            Messages::RoomName(id, name) => {
                if let Some(room) = self.rooms.get_mut(&id) {
                    room.name = name;
                }
            }
            Messages::BackFill(id) => {
                let room = self.rooms.entry(id.clone()).or_default();
                room.messages.loading = true;
                let client = self.client.clone();
                let token = match room.messages.end.clone() {
                    Some(end) => end,
                    None => client
                        .get_joined_room(&id)
                        .unwrap()
                        .last_prev_batch()
                        .unwrap_or_else(|| self.sync_token.clone()),
                };
                return async move {
                    let mut request = MessageRequest::backward(&id, &token);
                    request.limit = matrix_sdk::uint!(30);
                    match client.room_messages(request).await {
                        Ok(response) => Messages::BackFilled(id, response),
                        Err(e) => Messages::LoginFailed(e.to_string()),
                    }
                }
                .into();
            }
            Messages::BackFilled(id, response) => {
                let room = self.rooms.get_mut(&id).unwrap();
                room.messages.loading = false;
                let events: Vec<AnyRoomEvent> = response
                    .chunk
                    .into_iter()
                    .filter_map(|e| e.deserialize().ok())
                    .chain(
                        response
                            .state
                            .into_iter()
                            .filter_map(|e| e.deserialize().ok().map(AnyRoomEvent::State)),
                    )
                    .collect();
                if let Some(start) = response.start {
                    room.messages.start = Some(start);
                }
                if let Some(end) = response.end {
                    room.messages.end = Some(end);
                }
                //println!("Events: {:#?}", events);
                //let commands: Vec<Command<_>> = events
                //    .iter()
                //    .filter_map(|e| e.image_url())
                //    .map(|url| {
                //        async {
                //            println!("URL Back: {}", url);
                //            Messages::FetchImage(url)
                //        }
                //        .into()
                //    })
                //    .collect();
                //room.messages.append(events);

                let commands: Vec<Command<_>> = events
                    .iter()
                    .filter_map(|event| {
                        if let AnyRoomEvent::Message(message_event) = event {
                            let message_event_clone = message_event.clone();
                            return Some(
                                    async move {
                                        Messages::RoomMessage(message_event_clone.to_owned())
                                    }
                                    .into(),
                                );
                        } else {
                            None
                        }
                    })
                    .collect();

                return Command::batch(commands);
            }
            Messages::FetchImage(url) => {
                let (server, path) = match parse_mxc(&url) {
                    Ok((server, path)) => (server, path),
                    Err(e) => return async move { Messages::LoginFailed(e.to_string()) }.into(),
                };
                let client = self.client.clone();
                return async move {
                    let request = ImageRequest::new(&path, &*server);
                    let response = client.send(request).await;
                    match response {
                        Ok(response) => Messages::FetchedImage(
                            url,
                            iced::image::Handle::from_memory(response.file),
                        ),
                        Err(e) => Messages::LoginFailed(e.to_string()),
                    }
                }
                .into();
            }
            Messages::FetchedImage(url, handle) => {
                self.images.insert(url, handle);
            }
            Messages::SelectRoom(id) => {
                self.selected = Some(id.clone());
                if self.rooms.get(&id).unwrap().messages.messages.is_empty() {
                    return async move { Messages::BackFill(id) }.into();
                }
            }
            Messages::SetMessage(message) => self.draft = message,
            Messages::SendMessage => {
                let selected = match self.selected.clone() {
                    Some(selected) => selected,
                    None => return Command::none(),
                };
                let draft = self.draft.clone();
                let client = self.client.clone();
                return Command::perform(
                    async move {
                        client
                            .room_send(
                                &selected,
                                AnyMessageEventContent::RoomMessage(
                                    MessageEventContent::text_plain(draft),
                                ),
                                None,
                            )
                            .await
                    },
                    |result| match result {
                        Ok(_) => Messages::SetMessage(String::new()),
                        Err(e) => Messages::LoginFailed(e.to_string()),
                    },
                );
            }
            _ => {}
        };

        Command::none()
    }

    pub fn view(&mut self) -> iced::Element<'_, Messages> {
        let theme = self.theme;
        let ims = &self.images;

        let selected_room = match self.selected {
            Some(ref selected) => match (
                self.rooms.get(selected),
                self.client.get_joined_room(selected),
            ) {
                (Some(room), Some(joined)) => Some((room, joined)),
                _ => None,
            },
            None => None,
        };

        let (mut dm_rooms, mut group_rooms) = partition_rooms(&self.rooms, &self.client);

        // Sort by Recent
        for list in [&mut dm_rooms, &mut group_rooms].iter_mut() {
            list.sort_unstable_by(|(_, a), (_, b)| {
                a.messages.updated.cmp(&b.messages.updated).reverse()
            })
        }

        // Make sure button handler list has appropriate length
        self.dm_buttons
            .resize_with(dm_rooms.len(), Default::default);
        self.group_buttons
            .resize_with(group_rooms.len(), Default::default);

        let button_generator = |button, idx: usize, rooms: &[(&RoomId, &RoomEntry)]| {
            if let Some((id, room)) = rooms.get(idx) {
                let mut image_handler = None;
                if let Some(ref url) = room.avatar {
                    if let Some(handle) = ims.get(url) {
                        image_handler = Some(handle.clone());
                    }
                }
                let mut button_content = Row::new();

                if let Some(handle) = image_handler {
                    button_content = button_content.push(
                        Image::new(handle.to_owned())
                            .width(iced::Length::FillPortion(1))
                            .height(iced::Length::Fill),
                    );
                }

                Some(
                    Button::new(
                        button,
                        button_content.push(
                            Text::new(if room.name.is_empty() {
                                "Empty Room"
                            } else {
                                &room.name
                            })
                            .width(iced::Length::FillPortion(4)),
                        ),
                    )
                    .width(iced::Length::Fill)
                    .style(theme)
                    .on_press(Messages::SelectRoom(id.to_owned().to_owned())),
                )
            } else {
                None
            }
        };

        let dm_buttons = self
            .dm_buttons
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, button)| button_generator(button, idx, &dm_rooms));

        let room_buttons = self
            .group_buttons
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, button)| button_generator(button, idx, &group_rooms));

        let mut room_scroll_view = Scrollable::new(&mut self.room_scroll).spacing(10);

        room_scroll_view =
            room_scroll_view.push(Container::new(Text::new("Direct messages")).padding(10));
        for button in dm_buttons.into_iter() {
            room_scroll_view = room_scroll_view.push(button);
        }
        room_scroll_view = room_scroll_view.push(Container::new(Text::new("Rooms")).padding(10));
        for button in room_buttons.into_iter() {
            room_scroll_view = room_scroll_view.push(button);
        }

        let mut message_col = Column::new().spacing(5).padding(5);

        if let Some((room, joined)) = selected_room {
            // Include user id or canonical alias in title when appropriate
            let title = if let Some(ref direct) = room.direct {
                format!("{} ({})", &room.name, direct)
            } else if let Some(ref alias) = room.alias {
                format!("{} ({})", &room.name, alias)
            } else {
                room.name.clone()
            };

            let mut title_row = Row::new().align_items(Align::Center);
            if let Some(handle) = room.avatar.as_deref().and_then(|a| ims.get(a)) {
                title_row = title_row.push(
                    Image::new(handle.to_owned())
                        .width(24.into())
                        .height(24.into()),
                );
            }

            message_col = message_col
                .push(title_row.push(Text::new(title).size(25)))
                .push(Rule::horizontal(2));

            let mut scroll = Scrollable::new(&mut self.message_scroll)
                .scrollbar_width(2)
                .spacing(4)
                .height(Length::Fill);
            // Backfill button or loading message
            let backfill: Element<_> = if room.messages.loading {
                Text::new("Loading...").into()
            } else if room.messages.has_beginning() {
                let creation = joined.create_content().unwrap();
                let mut col =
                    Column::new().push(Text::new("This is the beginning of room history"));
                if let Some(prevous) = creation.predecessor {
                    col = col.push(
                        Button::new(&mut self.backfill_button, Text::new("Go to older version"))
                            .on_press(Messages::SelectRoom(prevous.room_id)),
                    );
                }
                col.into()
            } else {
                Button::new(&mut self.backfill_button, Text::new("Load more messages"))
                    .on_press(Messages::BackFill(self.selected.clone().unwrap()))
                    .into()
            };
            scroll = scroll.push(Container::new(backfill).width(Length::Fill).center_x());
            // mxid of most recent sender
            let mut last_sender: Option<UserId> = None;
            // Messages

            println!("View Message List: {:#?}", room.message_list);
            for message in room.message_list.iter() {
                let mut message_container = Column::new();

                let sender = message.sender.clone();

                if last_sender != Some(sender.clone()) {
                    last_sender = Some(sender.clone());

                    let user_details = get_sender_details(sender, joined.clone());
                    let mut user_row = Row::new();
                    match user_details.1 {
                        Some(image) => match ims.get(&image) {
                            Some(image_handle) => {
                                user_row = user_row.push(Image::new(image_handle.to_owned()));
                            }
                            None => {}
                        },
                        None => {}
                    }

                    message_container =
                        message_container.push(user_row.push(Text::new(user_details.0)));
                }
                match message.content.clone() {
                    MessageEventContent::Audio(_) => {
                        println!("Audio Message");
                    }
                    MessageEventContent::Emote(_) => {}
                    MessageEventContent::File(_) => {}
                    MessageEventContent::Image(_) => {}
                    MessageEventContent::Location(_) => {}
                    MessageEventContent::Notice(_) => {}
                    MessageEventContent::ServerNotice(_) => {}
                    MessageEventContent::Text(text) => {
                        message_container = message_container.push(Text::new(&text.body));
                    }
                    MessageEventContent::Video(_) => {}
                    MessageEventContent::VerificationRequest(_) => {}
                    _ => {
                        println!("Unknown message type");
                    }
                }

                scroll = scroll.push(message_container);
            }

            //for event in room.messages.messages.iter() {
            //    #[allow(clippy::single_match)]
            //    match event {
            //        AnyRoomEvent::Message(AnyMessageEvent::RoomMessage(message)) => {
            //            // Display sender if message is from new sender
            //            if last_sender.as_ref() != Some(&message.sender) {
            //                last_sender = Some(message.sender.clone());

            //                let sender_image = match block_on(async {
            //                    joined.get_member(&message.sender).await.unwrap()
            //                }) {
            //                    Some(member) => {
            //                        let mem = member.clone();
            //                        mem.avatar_url().take().to_owned().clone()
            //                    }
            //                    None => None,
            //                };

            //                if let Some(image_url) = &sender_image {
            //                    //println!("URL: {}", image_url);
            //                }

            //                sender = match block_on(async {
            //                    joined.get_member(&message.sender).await.unwrap()
            //                }) {
            //                    Some(member) => member.name().to_owned(),
            //                    None => message.sender.to_string(),
            //                };

            //                scroll = scroll
            //                    .push(iced::Space::with_height(4.into()))
            //                    .push(Text::new(&sender).color([0.0, 0.0, 1.0]));
            //            }
            //            let content: Element<_> = match &message.content {
            //                MessageEventContent::Audio(audio) => {
            //                    Text::new(format!("Audio message: {}", audio.body))
            //                        .color([0.2, 0.2, 0.2])
            //                        .width(Length::Fill)
            //                        .into()
            //                }
            //                MessageEventContent::Emote(emote) => {
            //                    Text::new(format!("* {} {}", sender, emote.body))
            //                        .width(Length::Fill)
            //                        .into()
            //                }
            //                MessageEventContent::File(file) => {
            //                    Text::new(format!("File '{}'", file.body))
            //                        .color([0.2, 0.2, 0.2])
            //                        .width(Length::Fill)
            //                        .into()
            //                }
            //                MessageEventContent::Image(image) => {
            //                    if let Some(ref url) = image.url {
            //                        match self.images.get(url) {
            //                            Some(handle) => Container::new(
            //                                Image::new(handle.to_owned())
            //                                    .width(800.into())
            //                                    .height(1200.into()),
            //                            )
            //                            .width(Length::Fill)
            //                            .into(),
            //                            None => {
            //                                Text::new("Image not loaded").width(Length::Fill).into()
            //                            }
            //                        }
            //                    } else {
            //                        Text::new("Encrypted images not supported yet")
            //                            .width(Length::Fill)
            //                            .into()
            //                    }
            //                }
            //                MessageEventContent::Notice(notice) => {
            //                    Text::new(&notice.body).width(Length::Fill).into()
            //                }
            //                MessageEventContent::ServerNotice(notice) => {
            //                    Text::new(&notice.body).width(Length::Fill).into()
            //                }
            //                MessageEventContent::Text(text) => {
            //                    Text::new(&text.body).width(Length::Fill).into()
            //                }
            //                MessageEventContent::Video(video) => {
            //                    Text::new(format!("Video: {}", video.body))
            //                        .color([0.2, 0.2, 0.2])
            //                        .into()
            //                }
            //                _ => Text::new("Unknown message type").into(),
            //            };
            //            let row = Row::new()
            //                .spacing(5)
            //                .push(content)
            //                .push(Text::new(format_systime(message.origin_server_ts)));
            //            scroll = scroll.push(row);
            //        }
            //        AnyRoomEvent::Message(AnyMessageEvent::RoomEncrypted(_encrypted)) => {
            //            scroll = scroll.push(Text::new("Encrypted event").color([0.3, 0.3, 0.3]));
            //        }
            //        AnyRoomEvent::RedactedMessage(_) => {
            //            scroll = scroll.push(Text::new("Deleted message").color([0.3, 0.3, 0.3]));
            //        }
            //        _ => (),
            //    }
            //}
            // Tombstone
            if let Some(tombstone) = joined.tombstone() {
                let text = Text::new(format!(
                    "This room has been upgraded to a new version: {}",
                    tombstone.body
                ));
                let button =
                    Button::new(&mut self.tombstone_button, Text::new("Go to upgraded room"))
                        .on_press(Messages::SelectRoom(tombstone.replacement_room));
                scroll = scroll.push(
                    Container::new(
                        Column::new()
                            .push(text)
                            .push(button)
                            .align_items(Align::Center),
                    )
                    .center_x()
                    .width(Length::Fill),
                );
            }
            message_col = message_col.push(scroll);
        } else {
            message_col = message_col.push(
                Container::new(Text::new("Select a room to start chatting"))
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        }

        message_col = message_col.push(
            Row::new()
                .spacing(5)
                .push(
                    TextInput::new(
                        &mut self.message_input,
                        "Write a message...",
                        &self.draft,
                        Messages::SetMessage,
                    )
                    .width(Length::Fill)
                    .padding(5)
                    .style(theme)
                    .on_submit(Messages::SendMessage),
                )
                .push(
                    Button::new(&mut self.send_button, Text::new("Send"))
                        .style(theme)
                        .on_press(Messages::SendMessage),
                ),
        );

        let room_list_view = Container::new(room_scroll_view)
            .padding(20)
            .height(Length::Fill)
            .width(Length::FillPortion(1));

        let message_view = Container::new(message_col)
            .height(Length::Fill)
            .width(Length::FillPortion(4))
            .style(Theme::DarkRoom);

        Container::new(Row::new().push(room_list_view).push(message_view))
            .height(Length::Fill)
            .style(self.theme)
            .into()
    }
}

fn format_systime(time: std::time::SystemTime) -> String {
    let offset = time::UtcOffset::try_current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let time = time::OffsetDateTime::from(time).to_offset(offset);
    let today = time::OffsetDateTime::now_utc().to_offset(offset).date();
    // Display
    if time.date() == today {
        time.format("%T")
    } else {
        time.format("%F %T")
    }
}
