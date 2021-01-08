use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Element, Column, Text, Align, Container, Length, Subscription, Row, Checkbox, Rule, Color, Space, button};
use iced::time;
use std::sync::Arc;
use std::time::Instant;
use iced::widget::checkbox::Style;
use crate::windows::window::WindowMessage;
use iced::widget::button::State;

#[derive(Debug, Clone)]
pub enum HomeMessage {

}


#[derive(Debug, Clone)]
pub struct Home {
    server: Box<dyn ComServer>,
    can_state: button::State,
    uds_state: button::State
}

impl Home {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        let mut ret = Self {
            server,
            can_state: button::State::default(),
            uds_state: button::State::default()
        };
        // To guarantee everything works as it should, home screen should have NO interfaces open
        ret.server.close_can_interface();
        ret.server.close_iso15765_interface();
        ret
    }

    pub fn update(&mut self, msg: HomeMessage) -> Option<WindowMessage> {
        None
    }

    pub fn view(&mut self) -> Element<WindowMessage> {
        let cap = self.server.get_capabilities();
        let contents = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .push(Text::new("Welcome to OpenVehicleDiag"))
        // Render contents of info panel
            .push(Rule::horizontal(8))
            .push(Text::new("Adapter Info:"))
            .push(Text::new(format!("Hardware API: {}", self.server.get_api())))
            .push(Text::new(format!("Hardware name: {} (FW Version {})", cap.get_name(), cap.get_device_fw_version())))
            .push(Text::new(format!("Hardware vendor: {}", cap.get_vendor())))
            .push(Text::new(format!("Library path: {} (Version {})", cap.get_lib_path(), cap.get_library_version())))
            .push(Text::new(format!("Supported protocols")))
            .push(
            Row::new().spacing(5)
                    .push(
                Column::new()
                        .push(Text::new("CAN"))
                        .push(Text::new("ISO-TP"))
                        .push(Text::new("ISO9141"))
                        .push(Text::new("ISO14230")))
                    .push(
                    Column::new()
                            .push(Home::gen_cap_contents(cap.support_can_fd()))
                            .push(Home::gen_cap_contents(cap.supports_iso15765()))
                            .push(Home::gen_cap_contents(cap.supports_iso9141()))
                            .push(Home::gen_cap_contents(cap.supports_iso14230())))
                .push(Space::with_width(Length::Units(50)))
                  .push(
                        Column::new()
                            .push(Text::new("J1850PWM"))
                            .push(Text::new("J1850VPW"))
                            .push(Text::new("DoIP")))
                .push(
                    Column::new()
                        .push(Home::gen_cap_contents(cap.supports_j1850pwm()))
                        .push(Home::gen_cap_contents(cap.supports_j1850vpw()))
                        .push(Home::gen_cap_contents(cap.supports_doip())))
            ).push( Column::new()
            .align_items(Align::Center)
            .spacing(5)
            .push(Text::new("Tools"))
            .push(button::Button::new(&mut self.can_state, Text::new("CAN Tracer")).on_press(WindowMessage::GoCanTracer))
            .push(button::Button::new(&mut self.uds_state, Text::new("UDS Scanner")).on_press(WindowMessage::GoUDS))
            );
        contents.into()
    }
}

impl<'a> Home {
    fn gen_cap_contents(cap: Capability) -> Element<'a, WindowMessage> {
        match cap {
            Capability::Yes => Text::new("Yes").color(Color::from_rgb8(0, 128, 0)),
            Capability::No => Text::new("No").color(Color::from_rgb8(128, 0, 0)),
            Capability::NA => Text::new("N/A").color(Color::from_rgb8(192, 192, 192)),
        }.into()
    }
}