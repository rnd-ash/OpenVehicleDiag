use iced::{Column, Subscription};

use crate::{
    commapi::comm_api::{ComServer, ISO15765Config},
    themes::{title_text, TitleSize},
    windows::diag_manual::DiagManualMessage,
};

use super::{DiagMessageTrait, SessionError, SessionMsg, SessionResult, SessionTrait};

#[derive(Debug, Clone, PartialEq)]
pub enum UDSDiagSessionMsg {
    Back,
}

impl DiagMessageTrait for UDSDiagSessionMsg {
    fn is_back(&self) -> bool {
        self == &UDSDiagSessionMsg::Back
    }
}

#[derive(Debug, Clone)]
pub struct UDSDiagSession {
    ecu: ISO15765Config,
    server: Box<dyn ComServer>,
}

impl UDSDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config) -> SessionResult<Self> {
        //Err(SessionError::Other("UDS Session is not yet implemented".into()))
        Ok(Self {
            ecu,
            server: comm_server,
        })
    }
}

impl SessionTrait for UDSDiagSession {
    type msg = UDSDiagSessionMsg;

    fn view(&mut self) -> iced::Element<Self::msg> {
        todo!()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<Self::msg> {
        todo!()
    }

    fn subscription(&self) -> iced::Subscription<Self::msg> {
        Subscription::none()
    }
}
