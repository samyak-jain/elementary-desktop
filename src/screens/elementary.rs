use iced::{executor, keyboard, Application, Command, Subscription};
use iced_native::Event;

use crate::{matrix::subscriber::MatrixSync, session::get_session};

use super::{HomePage, LoginPage, Messages, VerifyPage};

pub enum Elementary {
    LoginPage(LoginPage),
    HomePage(HomePage),
    VerifyPage(VerifyPage),
}

impl Application for Elementary {
    type Executor = executor::Default;

    type Message = Messages;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let session_result = get_session();

        match session_result {
            Ok(Some(session)) => {
                let command = Command::perform(
                    async move { crate::matrix::login::restore_login(session).await },
                    |result| match result {
                        Ok((client, session)) => Self::Message::LoginResult(client, session),
                        Err(e) => Self::Message::LoginFailed(e.to_string()),
                    },
                );

                (Elementary::LoginPage(LoginPage::default()), command)
            }
            _ => (Elementary::LoginPage(LoginPage::default()), Command::none()),
        }
    }

    fn title(&self) -> String {
        String::from("Matrix")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Elementary::LoginPage(_) => iced_native::subscription::events_with(
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
            ),
            Elementary::HomePage(home) => {
                MatrixSync::subscription(home.client.clone()).map(Self::Message::Sync)
            }
            Elementary::VerifyPage(verify) => {
                MatrixSync::subscription(verify.client.clone()).map(Self::Message::Sync)
            }
        }
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match self {
            Elementary::LoginPage(login) => {
                let (command, page_to_move) = login.update(message);
                if let Some(page) = page_to_move {
                    *self = page;
                }

                command
            }
            Elementary::HomePage(home) => home.update(message),
            Elementary::VerifyPage(verify) => {
                let (command, page_to_move) = verify.update(message);
                if let Some(page) = page_to_move {
                    *self = page;
                }

                command
            }
        }
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        match self {
            Elementary::LoginPage(login) => login.view(),
            Elementary::HomePage(home) => home.view(),
            Elementary::VerifyPage(verify) => verify.view(),
        }
    }
}
