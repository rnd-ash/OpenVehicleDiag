use crate::commapi::{comm_api::{Capability, ComServer, ISO15765Config}, protocols::{ProtocolServer, obd2::ObdServer}};
use crate::commapi::protocols::vin::Vin;
use crate::themes::{button_outlined, text, title_text, ButtonType, TextType, TitleSize};
use iced::{button, Align, Button, Column, Element, Length, Row, Space, Text};

#[derive(Debug, Clone)]
pub enum OBDMessage {
    InitOBD_IsoTP,
}

#[derive(Debug, Clone)]
pub struct OBDHome {
    server: Box<dyn ComServer>,
    kline_state: button::State,
    can_state: button::State,
}

impl OBDHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            kline_state: Default::default(),
            can_state: Default::default(),
        }
    }

    pub fn update(&mut self, msg: &OBDMessage) -> Option<OBDMessage> {
        match msg {
            OBDMessage::InitOBD_IsoTP => {
                let cfg = ISO15765Config {
                    baud: 500_000,
                    send_id: 0x07DF,
                    recv_id: 0x07E8,
                    block_size: 8,
                    sep_time: 20,

                };
                ObdServer::start_diag_session(self.server.clone(), &cfg, None);
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
                    .on_press(OBDMessage::InitOBD_IsoTP)
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
        c.into()
    }
}
