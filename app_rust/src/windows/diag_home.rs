use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Element, Column, Text, Align, Length, Row, Rule, Space, button};
use crate::windows::window::WindowMessage;
use crate::themes::{title_text, text, TextType, button_outlined, ButtonType, TitleSize};


#[derive(Debug, Clone)]
pub enum DiagHomeMessage {
    //Scanner(DiagScanMessage),
    //ManualSession(ManualDiagMessage),
}


#[derive(Debug, Clone)]
pub struct DiagHome {
    server: Box<dyn ComServer>,
}

impl DiagHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server
        }
    }

    pub fn update(&mut self, _msg: &DiagHomeMessage) -> Option<WindowMessage> {
        None
    }

    pub fn view(&mut self) -> Element<WindowMessage> {
        todo!()
    }
}