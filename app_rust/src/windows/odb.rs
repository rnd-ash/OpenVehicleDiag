use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Element, Column, Text, Align, Container, Length, Subscription, Row, Checkbox, Rule, Color, Space, button, Button};
use iced::time;
use std::sync::Arc;
use std::time::Instant;
use iced::widget::checkbox::Style;
use crate::windows::window::WindowMessage;
use iced::widget::button::State;

#[derive(Debug, Clone)]
pub enum ODBMessage {

}


#[derive(Debug, Clone)]
pub struct ODBHome {
    server: Box<dyn ComServer>,
    kline_state: button::State,
    can_state: button::State,
}

impl ODBHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        let mut ret = Self {
            server,
            kline_state: Default::default(),
            can_state: Default::default(),
        };
        ret
    }

    pub fn update(&mut self, msg: &ODBMessage) -> Option<ODBMessage> {
        None
    }

    pub fn view(&mut self) -> Element<ODBMessage> {
        let odb_btn = Button::new(&mut self.kline_state, Text::new("K-Line not implemented")); // TODO Add K-LINE ODB
        let can_btn = match self.server.get_capabilities().supports_iso15765() {
            Capability::Yes => Button::new(&mut self.can_state, Text::new("ODB over CANBUS")),
            _ => Button::new(&mut self.can_state, Text::new("No CANBUS Support on adapter"))
        };


        let mut c = Column::new()
            .push(Text::new("ODB Diagnostics"))
            .push(Space::with_height(Length::Units(10)))
            .push(Row::new()
                .push(odb_btn)
                .push(can_btn))
            .align_items(Align::Center);
        c.into()
    }
}