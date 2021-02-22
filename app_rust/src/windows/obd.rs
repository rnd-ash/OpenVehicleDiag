use crate::commapi::comm_api::{Capability, ComServer};
use crate::commapi::protocols::obd2::{Service01, Service09};
use crate::commapi::protocols::vin::Vin;
use crate::themes::{button_outlined, text, title_text, ButtonType, TextType, TitleSize};
use iced::{button, Align, Button, Column, Element, Length, Row, Space, Text};

#[derive(Debug, Clone)]
pub enum OBDMessage {
    InitOBD,
}

#[derive(Debug, Clone)]
pub struct OBDHome {
    server: Box<dyn ComServer>,
    kline_state: button::State,
    can_state: button::State,
    vin: Option<Vin>,
    s1: Option<Service01>,
    s9: Option<Service09>,
}

impl OBDHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            kline_state: Default::default(),
            can_state: Default::default(),
            vin: None,
            s1: None,
            s9: None,
        }
    }

    pub fn update(&mut self, msg: &OBDMessage) -> Option<OBDMessage> {
        match msg {
            OBDMessage::InitOBD => {
                if let Ok(s1) = Service01::init(&mut self.server, true) {
                    self.s1 = Some(s1)
                }
                if let Ok(s9) = Service09::init(&mut self.server, true) {
                    if let Ok(vin) = s9.get_vin(&mut self.server, true) {
                        self.vin = Some(vin);
                    }
                    self.s9 = Some(s9)
                }
            }
        }
        None
    }

    pub fn view(&mut self) -> Element<OBDMessage> {
        let obd_btn = button_outlined(
            &mut self.kline_state,
            "K-Line not implemented",
            ButtonType::Danger,
        ); // TODO Add K-LINE OBD
        let can_btn = match self.server.get_capabilities().supports_iso15765() {
            Capability::Yes => {
                button_outlined(&mut self.can_state, "OBD over CANBUS", ButtonType::Danger)
                    .on_press(OBDMessage::InitOBD)
            }
            _ => Button::new(
                &mut self.can_state,
                Text::new("No CANBUS Support on adapter"),
            ),
        };

        let mut c = Column::new()
            .padding(10)
            .spacing(10)
            .push(title_text("OBD Diagnostics", TitleSize::P2))
            .push(Space::with_height(Length::Units(10)))
            .push(
                Row::new()
                    .padding(10)
                    .spacing(10)
                    .push(obd_btn)
                    .push(can_btn),
            )
            .align_items(Align::Center);

        if let Some(vin) = &self.vin {
            c = c.push(Text::new(format!("VIN: {}", vin.raw)));
            c = c.push(Text::new(format!("Year: {}", vin.year)));
            c = c.push(Text::new(format!("Manufacture: {}", vin.manufacture_name)));
            c = c.push(Text::new(format!("Location: {}", vin.manufacture_location)));
        }
        c = c.push(Space::with_height(Length::Units(10)));

        c = c.push(title_text("Supported Services", TitleSize::P4));

        let s01 = if self.s1.is_some() { "Yes" } else { "No" };
        let s09 = if self.s1.is_some() { "Yes" } else { "No" };

        c = c.push(text(
            format!("Service 01: {}", s01).as_str(),
            TextType::Normal,
        ));
        c = c.push(text(
            format!("Service 09: {}", s09).as_str(),
            TextType::Normal,
        ));

        if let Some(service_01) = self.s1 {
            let mut pid_row = Row::new();
            pid_row = pid_row.push(Text::new("Service 01 supported PIDS: "));
            for pid in service_01.get_supported_pids() {
                pid_row = pid_row.push(Text::new(format!("{:02X} ", pid)));
            }
            c = c.push(pid_row);
        }
        c.width(Length::Fill).align_items(Align::Center).into()
    }
}
