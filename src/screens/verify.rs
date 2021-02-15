use iced::{Button, Column, Command, Container, Font, Length, Row, Text};

use super::{elementary::Elementary, HomePage, LoginPage, Messages, VerifyPage};

struct EmojiHandler<T> {
    stride: usize,
    data: Vec<T>,
}

impl<T> EmojiHandler<T> {
    fn get(&self, row: usize, col: usize) -> Option<&T> {
        let index = row * self.stride + col;
        if index >= self.data.len() {
            None
        } else {
            Some(&self.data[index])
        }
    }
}

const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("/usr/share/fonts/Unifont/Unifont.ttf"),
};

impl VerifyPage {
    pub fn update(&mut self, message: super::Messages) -> (Command<Messages>, Option<Elementary>) {
        match message {
            Messages::Sync(event) => match event {
                crate::matrix::subscriber::MatrixEvents::Room(_) => {}
                crate::matrix::subscriber::MatrixEvents::ToDevice(device_event) => {
                    match device_event {
                        matrix_sdk::events::AnyToDeviceEvent::Dummy(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::RoomKey(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::RoomKeyRequest(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::ForwardedRoomKey(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::KeyVerificationRequest(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::KeyVerificationStart(key_event) => {
                            let client = self.client.clone();
                            return (
                                async move {
                                    let sas = client
                                        .get_verification(&key_event.content.transaction_id)
                                        .await
                                        .expect("Sas object wasn't created");

                                    match sas.accept().await {
                                        Ok(_) => Messages::LoginFailed(String::from("")),
                                        Err(err) => Messages::LoginFailed(err.to_string()),
                                    }
                                }
                                .into(),
                                None,
                            );
                        }
                        matrix_sdk::events::AnyToDeviceEvent::KeyVerificationCancel(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::KeyVerificationAccept(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::KeyVerificationKey(key_event) => {
                            let client = self.client.clone();
                            return (
                                async move {
                                    let sas = client
                                        .get_verification(&key_event.content.transaction_id)
                                        .await
                                        .expect("Sas object wasn't created");

                                    if let Some(emoji) = sas.emoji() {
                                        Messages::SetVerification(emoji, sas)
                                    } else {
                                        Messages::LoginFailed(String::from("No Emoji"))
                                    }
                                }
                                .into(),
                                None,
                            );
                        }
                        matrix_sdk::events::AnyToDeviceEvent::KeyVerificationMac(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::RoomEncrypted(_) => {}
                        matrix_sdk::events::AnyToDeviceEvent::Custom(_) => {}
                    };
                }
            },
            Messages::SetVerification(verification_content, sas) => {
                self.verification_emoji = verification_content;
                self.sas = Some(sas);
            }
            Messages::VerificationConfirm => {
                if let Some(sas) = self.sas.clone() {
                    return (
                        async move {
                            match sas.confirm().await {
                                Ok(_) => Messages::GoHome,
                                Err(_) => Messages::GoBack,
                            }
                        }
                        .into(),
                        None,
                    );
                }
            }
            Messages::VerificationCancel => {
                if let Some(sas) = self.sas.clone() {
                    return (
                        async move {
                            sas.cancel().await.unwrap();
                            Messages::GoBack
                        }
                        .into(),
                        None,
                    );
                }
            }
            Messages::GoHome => {
                return (
                    Command::none(),
                    Some(Elementary::HomePage(HomePage::new(
                        self.client.clone(),
                        self.session.clone(),
                    ))),
                );
            }
            Messages::GoBack => {
                return (
                    Command::none(),
                    Some(Elementary::LoginPage(LoginPage::default())),
                );
            }
            _ => {}
        };

        (Command::none(), None)
    }

    pub fn view(&mut self) -> iced::Element<'_, super::Messages> {
        let emoji_grid_stride = 4;

        let emoji_handler = EmojiHandler {
            stride: emoji_grid_stride,
            data: self.verification_emoji.clone(),
        };

        let num_rows = if emoji_handler.data.len() % emoji_grid_stride == 0 {
            emoji_handler.data.len() / emoji_grid_stride
        } else {
            emoji_handler.data.len() / emoji_grid_stride + 1
        };

        let mut emoji = Column::new().padding(70).spacing(20);

        for row in 0..num_rows {
            let mut emoji_row = Row::new().spacing(20);

            for col in 0..emoji_grid_stride {
                if let Some((emoji, text)) = emoji_handler.get(row, col) {
                    let emoji_view = Column::new()
                        .push(Text::new(emoji.to_owned().to_owned()).font(ICONS))
                        .push(Text::new(text.to_owned().to_owned()));

                    emoji_row = emoji_row.push(emoji_view);
                };
            }

            emoji = emoji.push(emoji_row);
        }

        let buttons = Row::new()
            .push(
                Button::new(&mut self.accept_button_state, Text::new("Accept"))
                    .style(self.theme)
                    .on_press(Messages::VerificationConfirm),
            )
            .push(
                Button::new(&mut self.cancel_button_state, Text::new("Cancel"))
                    .style(self.theme)
                    .on_press(Messages::VerificationCancel),
            );

        Container::new(
            Column::new()
                .push(
                    Container::new(emoji)
                        .height(Length::FillPortion(3))
                        .width(Length::Fill)
                        .style(self.theme)
                        .center_y(),
                )
                .push(
                    Container::new(buttons)
                        .height(Length::FillPortion(1))
                        .width(Length::Fill)
                        .style(self.theme)
                        .center_x(),
                ),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .style(self.theme)
        .into()
    }
}
