use std::{collections::BTreeMap, convert::TryInto};

use crate::{database::connection::establish_connection, matrix::room::RoomEntry};

use super::{HomePage, LoginPage, Messages};
use iced::{Align, Button, Column, Command, Container, Length, Row, Svg, Text, TextInput};
use num_traits::FromPrimitive;

#[derive(FromPrimitive)]
pub enum TextBoxes {
    Homeserver = 0,
    Username,
    Password,
}

impl LoginPage {
    pub fn set_focus(&mut self, to_focus: TextBoxes) {
        self.homerserver_state.unfocus();
        self.username_state.unfocus();
        self.password_state.unfocus();
        match to_focus {
            TextBoxes::Homeserver => self.homerserver_state.focus(),
            TextBoxes::Username => self.username_state.focus(),
            TextBoxes::Password => self.password_state.focus(),
        }
    }

    pub fn update(&mut self, message: Messages) -> (Command<Messages>, Option<HomePage>) {
        let textboxes = [
            &self.homerserver_state,
            &self.username_state,
            &self.password_state,
        ];

        match message {
            Messages::HomeserverChanged(input) => self.homeserver_url = input,
            Messages::UsernameChanged(input) => self.username = input,
            Messages::PasswordChanged(input) => self.password = input,
            Messages::FocusNext => {
                let focus_index = textboxes.iter().position(|textbox| textbox.is_focused());
                if let Some(unwrapped_focus_index) = focus_index {
                    if unwrapped_focus_index < textboxes.len() {
                        if let Some(textbox) =
                            FromPrimitive::from_i32((unwrapped_focus_index + 1).try_into().unwrap())
                        {
                            self.set_focus(textbox);
                        }
                    }
                }
            }
            Messages::FocusPrev => {
                let focus_index = textboxes.iter().position(|textbox| textbox.is_focused());
                if let Some(unwrapped_focus_index) = focus_index {
                    if 0 < unwrapped_focus_index {
                        if let Some(textbox) =
                            FromPrimitive::from_i32((unwrapped_focus_index - 1).try_into().unwrap())
                        {
                            self.set_focus(textbox);
                        }
                    }
                }
            }
            Messages::Submit => {
                let homeser = self.homeserver_url.clone();
                let user = self.username.clone();
                let pass = self.password.clone();

                return (
                    Command::perform(
                        async move { crate::matrix::login::login(&homeser, &user, &pass).await },
                        |result| match result {
                            Ok((client, session)) => Messages::LoginResult(client, session.into()),
                            Err(e) => Messages::LoginFailed(e.to_string()),
                        },
                    ),
                    None,
                );
            }
            Messages::LoginResult(client, session) => {
                println!("Logged In, {:#?}", client);
                let mut commands: Vec<Command<Messages>> = Vec::new();
                for room in client.joined_rooms().into_iter() {
                    let room = std::sync::Arc::new(room);
                    let r = room.clone();
                    let command: Command<_> = async move {
                        let entry = RoomEntry::from_sdk(&r).await;
                        Messages::ResetRoom(r.room_id().to_owned(), entry)
                    }
                    .into();
                    if let Some(url) = room.avatar_url() {
                        commands.push(async { Messages::FetchImage(url) }.into())
                    }
                    commands.push(command);
                }
                return (
                    Command::batch(commands),
                    Some(HomePage {
                        client,
                        session,
                        conn: establish_connection(),
                        rooms: Default::default(),
                        selected: None,
                        sync_token: Default::default(),
                        images: Default::default(),
                        theme: Default::default(),
                        dm_buttons: Default::default(),
                        group_buttons: Default::default(),
                        room_scroll: Default::default(),
                        message_scroll: Default::default(),
                        backfill_button: Default::default(),
                        tombstone_button: Default::default(),
                        message_input: Default::default(),
                        draft: Default::default(),
                        send_button: Default::default(),
                    }),
                );
            }
            Messages::LoginFailed(e) => println!("Login Failed, {:#?}", e),
            _ => (),
        };

        (Command::none(), None)
    }

    pub fn view(&mut self) -> iced::Element<'_, Messages> {
        let svg = Svg::from_path(format!(
            "{}/src/resources/matrix-logo.svg",
            env!("CARGO_MANIFEST_DIR")
        ));

        let matrix_logo = Container::new(svg)
            .padding(50)
            .center_x()
            .center_y()
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .style(self.theme);

        let login_form = Container::new(
            Column::new()
                .padding(70)
                .spacing(20)
                .push(
                    TextInput::new(
                        &mut self.homerserver_state,
                        "Enter Homeserver URL...",
                        &self.homeserver_url,
                        Messages::HomeserverChanged,
                    )
                    .size(15)
                    .padding(12)
                    .style(self.theme),
                )
                .push(
                    TextInput::new(
                        &mut self.username_state,
                        "Enter Username...",
                        &self.username,
                        Messages::UsernameChanged,
                    )
                    .size(15)
                    .padding(12)
                    .style(self.theme),
                )
                .push(
                    TextInput::new(
                        &mut self.password_state,
                        "Enter Password...",
                        &self.password,
                        Messages::PasswordChanged,
                    )
                    .password()
                    .size(15)
                    .padding(12)
                    .style(self.theme),
                )
                .push(
                    Button::new(
                        &mut self.button_state,
                        Text::new("Login").horizontal_alignment(iced::HorizontalAlignment::Center),
                    )
                    .width(Length::Fill)
                    .style(self.theme)
                    .on_press(Messages::Submit),
                ),
        )
        .height(Length::Fill)
        .width(Length::FillPortion(1))
        .style(self.theme)
        .center_y();

        Container::new(Row::new().push(matrix_logo).push(login_form))
            .height(Length::Fill)
            .style(self.theme)
            .into()
    }
}
