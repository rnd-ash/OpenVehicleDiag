use iced::Column;

use crate::{commapi::{comm_api::{ComServer, ISO15765Config}, protocols::{ProtocolServer, kwp2000::KWP2000ECU}}, themes::{ButtonType, TextType, TitleSize, button_outlined, text, title_text}, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionMsg, SessionTrait};


#[derive(Debug, Clone)]
pub enum KWP2000DiagSessionMsg {
    ConnectECU,
    DisconnectECU,
}

impl DiagMessageTrait for KWP2000DiagSessionMsg {

}

#[derive(Debug, Clone)]
pub struct KWP2000DiagSession {
    ecu: ISO15765Config,
    server: Box<dyn ComServer>,
    status: String,
    connect_btn: iced::button::State,
    disconnect_btn: iced::button::State,
    diag_server: Option<KWP2000ECU>
}

impl KWP2000DiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config) -> Self {
        Self {
            ecu,
            server: comm_server,
            status: "".into(),
            connect_btn: Default::default(),
            disconnect_btn: Default::default(),
            diag_server: None,
        }
    }
}

impl SessionTrait for KWP2000DiagSession {
    type msg = KWP2000DiagSessionMsg;


    fn view(&mut self) -> iced::Element<Self::msg> {
        let mut ui = Column::new()
            .push(title_text("KWP2000 diagnostic session", TitleSize::P3));


        let display_btn = if let Some(ref s) = self.diag_server {
            if s.is_in_diag_session() {
                button_outlined(&mut self.disconnect_btn, "Disconnect ECU", ButtonType::Warning).on_press(KWP2000DiagSessionMsg::DisconnectECU)
            } else {
                button_outlined(&mut self.disconnect_btn, "Connect ECU", ButtonType::Primary).on_press(KWP2000DiagSessionMsg::ConnectECU)
            }
        } else {
            button_outlined(&mut self.disconnect_btn, "Connect ECU", ButtonType::Primary).on_press(KWP2000DiagSessionMsg::ConnectECU)
        };


        ui = ui.push(display_btn);
        ui = ui.push(text(&self.status, TextType::Danger));
        ui.into()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<SessionMsg> {
        match msg {
            KWP2000DiagSessionMsg::ConnectECU => {
                match KWP2000ECU::start_diag_session(self.server.clone(), &self.ecu) {
                    Ok(server) => self.diag_server = Some(server),
                    Err(e) => {
                        self.status = e.get_text()
                    }
                }
            },
            KWP2000DiagSessionMsg::DisconnectECU => {
                if let Some(ref mut server) = self.diag_server {
                    server.exit_diag_session()
                }
                self.diag_server.take();
            },

            _ =>{}
        }
        None
    }
}