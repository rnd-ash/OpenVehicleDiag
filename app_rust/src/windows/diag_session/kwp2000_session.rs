use iced::Column;

use crate::{commapi::comm_api::{ComServer, ISO15765Config}, themes::{TitleSize, title_text}, windows::diag_manual::DiagManualMessage};

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
}

impl KWP2000DiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config) -> Self {
        Self {
            ecu,
            server: comm_server,
            status: "".into()
        }
    }
}

impl SessionTrait for KWP2000DiagSession {
    type msg = KWP2000DiagSessionMsg;


    fn view(&mut self) -> iced::Element<SessionMsg> {
        Column::new()
            .push(title_text("KWP2000 diagnostic session", TitleSize::P3))
        .into()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<SessionMsg> {
        todo!()
    }
}