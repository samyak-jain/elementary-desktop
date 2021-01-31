use super::{LoginPage, Messages};
use iced::{Button, Column, Container, Length, Row, Svg, Text, TextInput};
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
                    Button::new(&mut self.button_state, Text::new("Login"))
                        .on_press(Messages::Submit),
                ),
        )
        .height(Length::Fill)
        .width(Length::FillPortion(1))
        .center_y();

        Container::new(Row::new().push(matrix_logo).push(login_form))
            .height(Length::Fill)
            .style(self.theme)
            .into()
    }
}
