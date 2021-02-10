use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Align, Column, Element, Length, Row, Rule, Space, Subscription, Text, button};
use crate::windows::window::WindowMessage;
use crate::themes::{title_text, text, TextType, button_outlined, ButtonType, TitleSize};
use super::{diag_manual::{self, DiagManual, DiagManualMessage}, diag_scanner::{DiagScanner, DiagScannerMessage}};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub (crate) struct VehicleECUList {
    pub (crate) vehicle_name: String,
    pub (crate) vehicle_year: u32,
    pub (crate) vehicle_brand: String,
    pub (crate) ecu_list: Vec<ECUDiagSettings>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ECUDiagSettings {
    pub (crate) name: String,
    pub (crate) send_id: u32,
    pub (crate) flow_control_id: u32,
    pub (crate) block_size: u32,
    pub (crate) sep_time_ms: u32,
    pub (crate) uds_support: bool,
    pub (crate) kwp_support: bool
}

impl ToString for ECUDiagSettings {
    fn to_string(&self) -> String {
        format!("{} (0x{:04X})", self.name, self.send_id)
    }
}

#[derive(Debug, Clone)]
pub enum DiagHomeMessage {
    Scanner(DiagScannerMessage),
    ManualSession(DiagManualMessage),
    LaunchScan,
    LaunchManual,
}


#[derive(Debug, Clone)]
pub struct DiagHome {
    server: Box<dyn ComServer>,
    manual_btn_state: iced::button::State,
    scan_btn_state: iced:: button::State,
    manual_mode: Option<DiagManual>,
    scan_mode: Option<DiagScanner>,

}

impl DiagHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            manual_btn_state: Default::default(),
            scan_btn_state: Default::default(),
            manual_mode: None,
            scan_mode: None,
        }
    }

    pub fn update(&mut self, msg: &DiagHomeMessage) -> Option<DiagHomeMessage> {
        match msg {
            DiagHomeMessage::LaunchScan => {
                if self.scan_mode.is_none() {
                    self.scan_mode = Some(DiagScanner::new(self.server.clone()));
                }
                None
            },
            DiagHomeMessage::LaunchManual => {
                if self.manual_mode.is_none() {
                    self.manual_mode = Some(DiagManual::new(self.server.clone()));
                }
                None
            },
            DiagHomeMessage::Scanner(s) => {
                match self.scan_mode {
                    Some(ref mut p) => p.update(s).map(DiagHomeMessage::Scanner),
                    None => None
                }
            },
            DiagHomeMessage::ManualSession(s) => {
                match self.manual_mode {
                    Some(ref mut p) => p.update(s).map(DiagHomeMessage::ManualSession),
                    None => None
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<DiagHomeMessage> {
        if let Some(ref manual) = self.manual_mode {
            manual.subscription().map(DiagHomeMessage::ManualSession)
        } else if let Some(ref scan) = self.scan_mode {
            //todo!()
            Subscription::none()
        } else {
            Subscription::none()
        }
    }

    pub fn view(&mut self) -> Element<DiagHomeMessage> {
        if let Some(ref mut manual) = self.manual_mode {
            manual.view().map(DiagHomeMessage::ManualSession)
        } else if let Some(ref mut scan) = self.scan_mode {
            scan.view().map(DiagHomeMessage::Scanner)
        } else {
            Column::new().padding(20).spacing(20).align_items(Align::Center)
            .push(title_text("Welcome to OVD diagnostics mode", TitleSize::P2))
            .push(Space::with_width(Length::Units(20)))
            .push(text(
                "In this mode, you can either get OVD to scan for diagnostic compatible \
                ECUs in your vehicle, or you can load a save file and see run diagnostic commands \
                using KWP2000, UDS or a custom JSON defined protocol"
                , TextType::Normal))
            .push(Space::with_width(Length::Units(5)))
            .push(text(
                "If you don't know what any of this means, please leave this mode, as you could \
                potentially damage your vehicle with this!"
                , TextType::Warning))
            .push(Space::with_width(Length::Units(5)))
            .push(text(
                "If you wish to scan your vehicle, press the scan button and let the automated \
                scan take place! - Remember to save your diagnostic scan file at the end of the \
                scan when prompted in order to make future scans much faster!"
                , TextType::Normal))
            .push(Space::with_width(Length::Units(5)))
            .push(text(
                "If you wish to interrogate a specific ECU in your vehicle and know what you are doing \
                with diagnostic protocols, press the manual mode to get started!"
                , TextType::Normal))
            .push(Row::new()
                .push(button_outlined(&mut self.scan_btn_state, "Scan my car", ButtonType::Success).on_press(DiagHomeMessage::LaunchScan))
                .push(Space::with_width(Length::Fill))
                .push(button_outlined(&mut self.manual_btn_state, "Launch manual mode", ButtonType::Warning).on_press(DiagHomeMessage::LaunchManual))
            )   
            .into()
        }
    }
}