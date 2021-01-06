use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Element, Column, Text, Align, Container, Length, Subscription, Row, Checkbox, Rule, Color, Space};
use iced::time;
use std::sync::Arc;
use std::time::Instant;
use iced::widget::checkbox::Style;

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

    fn gen_cap_contents(&self, cap: Capability) -> Element<HomeMessage> {
        match cap {
            Capability::Yes => Text::new("Yes").color(Color::from_rgb8(0, 128, 0)),
            Capability::No => Text::new("No").color(Color::from_rgb8(128, 0, 0)),
            Capability::NA => Text::new("N/A").color(Color::from_rgb8(192, 192, 192)),
        }.into()
    }

    pub fn view(&mut self) -> Element<HomeMessage> {
        let cap = self.server.get_capabilities();
        let contents = Column::new()
            .padding(10)
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new("Welcome to OpenVehicleDiag"))
            .push(Text::new(format!("Battery voltage is {}V", self.v_batt)))
        // Render contents of info panel
            .push(Rule::horizontal(8))
            .push(Text::new("Adapter Info:"))
            .push(Text::new(format!("Hardware API: {}", self.server.get_api())))
            .push(Text::new(format!("Hardware name: {} (FW Version {})", cap.get_name(), cap.get_device_fw_version())))
            .push(Text::new(format!("Hardware vendor: {}", cap.get_vendor())))
            .push(Text::new(format!("Library path: {} (Version {})", cap.get_lib_path(), cap.get_library_version())))
            .push(Text::new(format!("Supported protocols")))
            .push(
            Row::new()
                    .push(
                Column::new()
                        .push(Text::new("CAN"))
                        .push(Text::new("ISO-TP"))
                        .push(Text::new("ISO9141"))
                        .push(Text::new("ISO14230")))
                    .push(
                    Column::new()
                            .push(self.gen_cap_contents(cap.support_can_fd()))
                            .push(self.gen_cap_contents(cap.supports_iso15765()))
                            .push(self.gen_cap_contents(cap.supports_iso9141()))
                            .push(self.gen_cap_contents(cap.supports_iso14230())))
                .push(Rule::vertical(5))
                  .push(
                        Column::new()
                            .push(Text::new("J1850PWM"))
                            .push(Text::new("J1850VPW"))
                            .push(Text::new("DoIP")))
                .push(
                    Column::new()
                        .push(self.gen_cap_contents(cap.supports_j1850pwm()))
                        .push(self.gen_cap_contents(cap.supports_j1850vpw()))
                        .push(self.gen_cap_contents(cap.supports_doip())))

            );

        Container::new(contents).center_y().center_x().into()
    }
}