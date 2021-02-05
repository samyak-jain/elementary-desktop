#![feature(destructuring_assignment)]

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate diesel;

use iced::{Application, Settings};
use screens::elementary::Elementary;

mod database;
mod matrix;
mod schema;
mod screens;
mod session;
mod theme;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    Elementary::run(Settings::default())
}
