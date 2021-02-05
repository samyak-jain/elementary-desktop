use std::collections::BTreeMap;

use diesel::SqliteConnection;
use iced::{button, text_input};
use matrix_sdk::{identifiers::RoomId, Client, Session};

use crate::{
    matrix::{room::RoomEntry, subscriber::MatrixEvents},
    theme::style,
};

use matrix_sdk::api::r0::message::get_message_events::Response as MessageResponse;

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
    theme: style::Theme,
    conn: SqliteConnection,
    client: Client,
    session: Session,
    rooms: BTreeMap<RoomId, RoomEntry>,
    selected: Option<RoomId>,
    sync_token: String,
    images: BTreeMap<String, iced::image::Handle>,
    dm_buttons: Vec<iced::button::State>,
    group_buttons: Vec<iced::button::State>,
    room_scroll: iced::scrollable::State,
    message_scroll: iced::scrollable::State,
    backfill_button: iced::button::State,
    tombstone_button: iced::button::State,
    message_input: iced::text_input::State,
    draft: String,
    send_button: iced::button::State,
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
    Sync(MatrixEvents),
    FetchImage(String),
    FetchedImage(String, iced::image::Handle),
    RoomName(RoomId, String),
    ResetRoom(RoomId, RoomEntry),
    BackFill(RoomId),
    BackFilled(RoomId, MessageResponse),
    SelectRoom(RoomId),
    SetMessage(String),
    SendMessage,
}
