use iced::{Container, Length, Row};

use super::{HomePage, Messages};

impl HomePage {
    pub fn view(&mut self) -> iced::Element<'_, Messages> {
        Container::new(Row::new()).height(Length::Fill).into()
    }
}
