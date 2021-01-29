use iced::Subscription;

use crate::{commapi::comm_api::{ComServer, ISO15765Config}, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionMsg, SessionTrait};

#[derive(Debug, Clone, PartialEq)]
pub enum CustomDiagSessionMsg {
    Back
}

impl DiagMessageTrait for CustomDiagSessionMsg {
    fn is_back(&self) -> bool {
        self == &CustomDiagSessionMsg::Back
    }
}

#[derive(Debug, Clone)]
pub struct CustomDiagSession {
    server: Box<dyn ComServer>,
    ecu: ISO15765Config
}

impl CustomDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config) -> Self {
        Self {
            ecu,
            server: comm_server
        }
    }
}

impl SessionTrait for CustomDiagSession {
    type msg = CustomDiagSessionMsg;

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