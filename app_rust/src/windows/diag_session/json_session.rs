use common::schema::OvdECU;
use iced::Subscription;

use crate::{commapi::comm_api::{ComServer, ISO15765Config}, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionMsg, SessionTrait};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonDiagSessionMsg {
    Back
}

impl DiagMessageTrait for JsonDiagSessionMsg {
    fn is_back(&self) -> bool {
        self == &JsonDiagSessionMsg::Back
    }
}

#[derive(Debug, Clone)]
pub struct JsonDiagSession {
    server: Box<dyn ComServer>,
    ecu: ISO15765Config,
    ecu_data: OvdECU,
}

impl JsonDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config, ecu_data: OvdECU) -> Self {
        Self {
            ecu,
            server: comm_server,
            ecu_data
        }
    }
}

impl SessionTrait for JsonDiagSession {
    type msg = JsonDiagSessionMsg;

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