#[macro_use]
extern crate druid;

use druid::{AppLauncher, PlatformError, WindowDesc};
use screens::{login::login_ui, LoginState};

mod matrix;
mod screens;

fn main() -> Result<(), PlatformError> {
    tracing_subscriber::fmt::init();
    AppLauncher::with_window(WindowDesc::new(login_ui)).launch(LoginState::default())?;
    Ok(())
}
