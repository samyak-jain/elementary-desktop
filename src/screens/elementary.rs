use iced::{executor, keyboard, Application, Command, Subscription};
use iced_native::Event;
use matrix_sdk::Session;
use num_traits::FromPrimitive;
use std::convert::TryInto;

use crate::database::connection::establish_connection;

use super::{HomePage, LoginPage, Messages};

pub enum Elementary {
    LoginPage(LoginPage),
    HomePage(HomePage),
}

impl Application for Elementary {
    type Executor = executor::Default;

    type Message = Messages;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let conn = establish_connection();
        let session_result = crate::database::session::get_session(&conn);

        match session_result {
            Ok(session) => {
                let command = Command::perform(
                    async move { crate::matrix::login::restore_login("", Session { ..session }).await },
                    |result| match result {
                        Ok((client, session)) => Self::Message::LoginResult(client, session),
                        Err(e) => Self::Message::LoginFailed(e.to_string()),
                    },
                );

                (Elementary::HomePage(HomePage::default()), Command::none())
            }
            Err(_) => (Elementary::LoginPage(LoginPage::default()), Command::none()),
        }
    }

    fn title(&self) -> String {
        String::from("Matrix")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(
            |event: iced_native::Event, _status| -> Option<Self::Message> {
                match event {
                    Event::Keyboard(keyboard_event) => match keyboard_event {
                        keyboard::Event::KeyPressed {
                            key_code: keyboard::KeyCode::Tab,
                            modifiers,
                        } => Some(if modifiers.shift {
                            Self::Message::FocusPrev
                        } else {
                            Self::Message::FocusNext
                        }),
                        _ => None,
                    },
                    _ => None,
                }
            },
        )
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match self {
            Elementary::LoginPage(login) => {
                let textboxes = [
                    &login.homerserver_state,
                    &login.username_state,
                    &login.password_state,
                ];

                match message {
                    Self::Message::HomeserverChanged(input) => login.homeserver_url = input,
                    Self::Message::UsernameChanged(input) => login.username = input,
                    Self::Message::PasswordChanged(input) => login.password = input,
                    Self::Message::FocusNext => {
                        let focus_index = textboxes.iter().position(|textbox| textbox.is_focused());
                        if let Some(unwrapped_focus_index) = focus_index {
                            if unwrapped_focus_index < textboxes.len() {
                                if let Some(textbox) = FromPrimitive::from_i32(
                                    (unwrapped_focus_index + 1).try_into().unwrap(),
                                ) {
                                    login.set_focus(textbox);
                                }
                            }
                        }
                    }
                    Self::Message::FocusPrev => {
                        let focus_index = textboxes.iter().position(|textbox| textbox.is_focused());
                        if let Some(unwrapped_focus_index) = focus_index {
                            if 0 < unwrapped_focus_index {
                                if let Some(textbox) = FromPrimitive::from_i32(
                                    (unwrapped_focus_index - 1).try_into().unwrap(),
                                ) {
                                    login.set_focus(textbox);
                                }
                            }
                        }
                    }
                    Self::Message::Submit => {
                        let homeser = login.homeserver_url.clone();
                        let user = login.username.clone();
                        let pass = login.password.clone();

                        return Command::perform(
                            async move { crate::matrix::login::login(&homeser, &user, &pass).await },
                            |result| match result {
                                Ok((client, session)) => {
                                    Self::Message::LoginResult(client, session)
                                }
                                Err(e) => Self::Message::LoginFailed(e.to_string()),
                            },
                        );
                    }
                    Self::Message::LoginResult(client, session) => {
                        println!("Logged In, {:#?}", client);
                        *self = Elementary::HomePage(HomePage {
                            client: Some(client),
                            ..HomePage::default()
                        })
                    }
                    Self::Message::LoginFailed(e) => println!("Login Failed, {:#?}", e),
                }
            }
            Elementary::HomePage(_) => {}
        }

        Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        match self {
            Elementary::LoginPage(login) => login.view(),
            Elementary::HomePage(home) => home.view(),
        }
    }
}
