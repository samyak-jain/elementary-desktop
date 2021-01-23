use druid::Lens;

pub mod login;

#[derive(Clone, Data, Lens, Default)]
pub struct LoginState {
    homeserver_url: String,
    username: String,
    password: String,
}

const TEXTBOX_WIDTH: f64 = 400.0;
const TEXTBOX_HEIGHT: f64 = 50.0;
const TEXTBOX_VERTICAL_SPACING: f64 = 50.0;
