use druid::{
    text::format::{Formatter, Validation, ValidationError},
    widget::{Button, Flex, Svg, SvgData, TextBox, Widget, WidgetExt},
};

use druid::UnitPoint;
use url::Url;

use crate::matrix::login::login;

use super::{LoginState, TEXTBOX_HEIGHT, TEXTBOX_VERTICAL_SPACING, TEXTBOX_WIDTH};

use futures::executor;

struct UrlFormatter;

impl Formatter<String> for UrlFormatter {
    fn format(&self, value: &String) -> String {
        String::from(value)
    }

    fn validate_partial_input(&self, _input: &str, _sel: &druid::text::Selection) -> Validation {
        Validation::success()
    }

    fn value(&self, input: &str) -> Result<String, ValidationError> {
        match Url::parse(input) {
            Ok(url) => Ok(url.to_string()),
            Err(err) => Err(ValidationError::new(err)),
        }
    }
}

pub fn login_ui() -> impl Widget<LoginState> {
    let matrix_logo = match include_str!("../resources/matrix-logo.svg").parse::<SvgData>() {
        Ok(svg) => svg,
        Err(_) => SvgData::default(), // TODO: Handle Error
    };

    let intro = Svg::new(matrix_logo).align_horizontal(UnitPoint::CENTER);

    let form = Flex::column()
        .with_child(
            TextBox::new()
                .with_placeholder("Enter Homeserver URL")
                .with_formatter(UrlFormatter)
                .lens(LoginState::homeserver_url)
                .fix_width(TEXTBOX_WIDTH)
                .fix_height(TEXTBOX_HEIGHT),
        )
        .with_spacer(TEXTBOX_VERTICAL_SPACING)
        .with_child(
            TextBox::new()
                .with_placeholder("Username")
                .lens(LoginState::username)
                .fix_width(TEXTBOX_WIDTH)
                .fix_height(TEXTBOX_HEIGHT),
        )
        .with_spacer(TEXTBOX_VERTICAL_SPACING)
        .with_child(
            TextBox::new()
                .with_placeholder("Password")
                .lens(LoginState::password)
                .fix_width(TEXTBOX_WIDTH)
                .fix_height(TEXTBOX_HEIGHT),
        )
        .with_spacer(TEXTBOX_VERTICAL_SPACING)
        .with_child(
            Button::new("Login")
                .on_click(|_ctx, data: &mut LoginState, _env| {
                    executor::block_on(login(&data.homeserver_url, &data.username, &data.password))
                        .unwrap();
                })
                .fix_width(TEXTBOX_WIDTH)
                .fix_height(TEXTBOX_HEIGHT),
        )
        .align_vertical(UnitPoint::CENTER)
        .align_horizontal(UnitPoint::CENTER);

    Flex::row()
        .main_axis_alignment(druid::widget::MainAxisAlignment::Center)
        .with_flex_child(intro, 1.0)
        .with_flex_child(form.expand(), 1.0)
}
