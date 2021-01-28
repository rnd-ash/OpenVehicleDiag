use crate::{commapi::comm_api::{ComServer, ISO15765Config}, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionMsg, SessionTrait};

#[derive(Debug, Clone)]
pub enum UDSDiagSessionMsg {

}

impl DiagMessageTrait for UDSDiagSessionMsg {

}

#[derive(Debug, Clone)]
pub struct UDSDiagSession {
    ecu: ISO15765Config,
    server: Box<dyn ComServer>
}


impl UDSDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config) -> Self {
        Self {
            ecu,
            server: comm_server
        }
    }
}

impl SessionTrait for UDSDiagSession {
    type msg = UDSDiagSessionMsg;


    fn view(&mut self) -> iced::Element<Self::msg> {
        todo!()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<SessionMsg> {
        todo!()
    }
}