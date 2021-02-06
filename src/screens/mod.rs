use std::collections::BTreeMap;

use diesel::SqliteConnection;
use iced::{button, text_input};
use matrix_sdk::{events::AnyMessageEvent, identifiers::RoomId, Client, Sas, Session};

use crate::{
    database::connection::establish_connection,
    matrix::{room::RoomEntry, subscriber::MatrixEvents},
    theme::style,
};

use matrix_sdk::api::r0::message::get_message_events::Response as MessageResponse;

pub mod elementary;
pub mod home;
pub mod login;
pub mod verify;

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

pub struct VerifyPage {
    theme: style::Theme,
    client: Client,
    session: Session,
    verification_emoji: Vec<(&'static str, &'static str)>,
    sas: Option<Sas>,
    accept_button_state: button::State,
    cancel_button_state: button::State,
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

impl HomePage {
    fn new(client: Client, session: Session) -> Self {
        HomePage {
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
    RoomMessage(AnyMessageEvent),
    Verification(Client, Session),
    SetVerification(Vec<(&'static str, &'static str)>, Sas),
    VerificationConfirm,
    VerificationCancel,
    GoHome,
    GoBack,
}
