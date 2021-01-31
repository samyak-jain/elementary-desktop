use diesel::SqliteConnection;
use iced::{button, text_input};
use matrix_sdk::{Client, Session};

use crate::{database::connection::establish_connection, theme::style};

pub mod elementary;
pub mod home;
pub mod login;

#[derive(Default)]
pub struct LoginPage {
    theme: style::Theme,
    homerserver_state: text_input::State,
    homeserver_url: String,
    username_state: text_input::State,
    username: String,
    password_state: text_input::State,
    password: String,
    button_state: button::State,
}

pub struct HomePage {
    conn: SqliteConnection,
    client: Option<Client>,
    session: Option<Session>,
}

impl Default for HomePage {
    fn default() -> Self {
        Self {
            conn: establish_connection(),
            client: None,
            session: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Messages {
    HomeserverChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    FocusNext,
    FocusPrev,
    Submit,
    LoginResult(Client, Session),
    LoginFailed(String),
}
