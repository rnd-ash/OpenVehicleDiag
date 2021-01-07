use crate::commapi::comm_api::{ComServer, Capability, CanFrame, FilterType};
use iced::{Element, Column, Text, Align, Container, Length, Subscription, Row, Checkbox, Rule, Color, Space, button};
use iced::time;
use std::sync::Arc;
use std::time::Instant;
use iced::widget::checkbox::Style;
use crate::windows::window::WindowMessage;
use iced::widget::button::State;
use crate::windows::home::HomeMessage;
use std::fs::FileType;
use std::collections::HashMap;
use iced::widget::pane_grid::TitleBar;

#[derive(Debug, Clone)]
pub enum UDSHomeMessage {
    LaunchManual,
    LaunchAutomatic
}


#[derive(Debug, Clone)]
pub struct UDSHome {
    server: Box<dyn ComServer>,
    auto_state: button::State,
    manual_state: button::State
}

impl<'a> UDSHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            auto_state: button::State::default(),
            manual_state: button::State::default()
        }
    }


    pub fn update(&mut self, msg: UDSHomeMessage) -> Option<UDSHomeMessage> {
        match msg {
            UDSHomeMessage::LaunchManual => {}
            UDSHomeMessage::LaunchAutomatic => {}
        }
        None
    }

    pub fn view(&mut self) -> Element<UDSHomeMessage> {
        Column::new()
            .push(Text::new("Welcome to the UDS Diagnostics page"))
            .push(Text::new("There are 2 modes of operation for this tool, please select wisely!"))
            .push(Row::new()
                      .push(button::Button::new(&mut self.auto_state, Text::new("Let the app decide how to scan my car")).on_press(UDSHomeMessage::LaunchAutomatic))
                      .push(Space::with_width(Length::Fill))
                      .push( button::Button::new(&mut self.manual_state, Text::new("I know what I'm doing.")).on_press(UDSHomeMessage::LaunchManual))
            )
            .into()
    }
}