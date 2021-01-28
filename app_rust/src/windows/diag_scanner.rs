use std::todo;

use iced::Element;

use crate::commapi::comm_api::ComServer;

#[derive(Debug, Clone)]
pub enum DiagScannerMessage {
}


#[derive(Debug, Clone)]
pub struct DiagScanner {
    server: Box<dyn ComServer>,
}

impl DiagScanner {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
        }
    }

    pub fn update(&mut self, msg: &DiagScannerMessage) -> Option<DiagScannerMessage> {
        None
    }

    pub fn view(&mut self) -> Element<DiagScannerMessage> {
        todo!()
    }
}