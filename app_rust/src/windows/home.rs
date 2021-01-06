use crate::commapi::comm_api::ComServer;
use iced::{Element, Column, Text, Align, Container, Length, Subscription};
use iced::time;
use std::sync::Arc;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum HomeMessage {
    ReadBatt(Instant)
}


#[derive(Debug, Clone)]
pub struct Home {
    server: Box<dyn ComServer>,
    v_batt: f32,
}

impl Home {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        let v = server.read_battery_voltage();
        Self {
            server,
            v_batt: v.unwrap_or(0.0)
        }
    }

    pub fn update(&mut self, msg: HomeMessage) {
        match msg {
            HomeMessage::ReadBatt(_) => {
                self.v_batt = self.server.read_battery_voltage().unwrap_or(self.v_batt)
            }
        }
    }

    pub fn subscribe(&self) -> Subscription<HomeMessage> {
        time::every(std::time::Duration::from_secs(2))
            .map(HomeMessage::ReadBatt)
    }

    pub fn view(&mut self) -> Element<HomeMessage> {
        let contents = Column::new()
            .push(Text::new("Welcome to OpenVehicleDiag"))
            .push(Text::new(format!("Battery voltage is {}V", self.v_batt)))
            .align_items(Align::Center);
        Container::new(contents).center_y().height(Length::Fill).into()
    }
}