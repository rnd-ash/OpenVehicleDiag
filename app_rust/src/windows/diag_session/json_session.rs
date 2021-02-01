use common::schema::{OvdECU, variant::{ECUVariantDefinition, ECUVariantPattern}};
use iced::{Align, Column, Container, Length, Row, Subscription};

use crate::{commapi::{comm_api::{ComServer, ISO15765Config}, protocols::{DiagProtocol, DiagServer, ProtocolServer, kwp2000::{KWP2000ECU, Service}}}, themes::{ButtonType, button_outlined, text, title_text}, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionError, SessionMsg, SessionResult, SessionTrait, log_view::{LogType, LogView}};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonDiagSessionMsg {
    ReadErrors,
    ClearErrors,
    Back
}

impl DiagMessageTrait for JsonDiagSessionMsg {
    fn is_back(&self) -> bool {
        self == &JsonDiagSessionMsg::Back
    }
}

#[derive(Debug, Clone)]
pub struct JsonDiagSession {
    server: DiagServer,
    ecu: ISO15765Config,
    ecu_text: (String, String),
    ecu_data: ECUVariantDefinition,
    pattern: ECUVariantPattern,
    log_view: LogView,
    clear_errors: iced::button::State,
    can_clear: bool,
    read_errors: iced::button::State
}

impl JsonDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config, ecu_data: OvdECU) -> SessionResult<Self> {
        match DiagServer::new(comm_server, &ecu, DiagProtocol::KWP2000) {
            Ok(mut server) => {
                let res = server.run_cmd(Service::ReadECUID, &[0x87], 500)?;
                let variant = (res[3] as u32) << 8 | (res[4] as u32);
                let ecu_variant = ecu_data.variants.into_iter().find(|x| {
                    x.clone().patterns.into_iter().any(|p| p.vendor_id == variant)
                });


                if let Some(v) = ecu_variant {
                    let pattern = v.clone().patterns.into_iter().find(|x| x.vendor_id == variant).unwrap();
                    println!("ECU Variant: {} (Vendor: {})", v.name, pattern.vendor);

                    Ok(Self {
                        ecu,
                        ecu_text: (ecu_data.name, ecu_data.description),
                        server,
                        ecu_data: v,
                        pattern,
                        log_view: LogView::new(),
                        read_errors: Default::default(),
                        clear_errors: Default::default(),
                        can_clear: false
                    })
                } else {
                    Err(SessionError::Other("Could not locate ECU variant".into()))
                }
            },
            Err(e) => {
                Err(SessionError::ServerError(e))
            }
        }
    }
}

impl SessionTrait for JsonDiagSession {
    type msg = JsonDiagSessionMsg;

    fn view(&mut self) -> iced::Element<Self::msg> {

        let mut btn_view = Column::new()
        .push(button_outlined(&mut self.read_errors, "Read errors", ButtonType::Primary).on_press(JsonDiagSessionMsg::ReadErrors))
        .width(Length::FillPortion(1));

        if self.can_clear {
            btn_view = btn_view.push(button_outlined(&mut self.clear_errors, "Clear errors", ButtonType::Primary).on_press(JsonDiagSessionMsg::ClearErrors))
        }


        Column::new().align_items(Align::Center)
            .push(title_text(format!("ECU: {} ({}). DiagVersion: {}, Vendor: {}", self.ecu_text.0, self.ecu_text.1, self.ecu_data.name, self.pattern.vendor).as_str(), crate::themes::TitleSize::P4))
            .push(
            Row::new()
                .push(btn_view)
                .push(Container::new(self.log_view.view()).width(Length::FillPortion(1))))
        .into()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<Self::msg> {
        self.log_view.clear_logs();
        match msg {
            JsonDiagSessionMsg::ReadErrors => {
                match self.server.read_errors() {
                    Ok(res) => {
                        if res.is_empty() {
                            self.log_view.add_msg("No ECU Errors 😊", LogType::Info);
                            self.can_clear = false;
                        } else {
                            self.log_view.add_msg(format!("Found {} errors 😢", res.len()), LogType::Warn);
                            for e in res {
                                let desc = self.ecu_data.errors.clone().into_iter().find(|x| x.error_name == e.error);
        
                                let err_txt = match desc {
                                    Some(d) =>  format!("{} - {}", e.error, d.description),
                                    None => format!("{} - Unknown", e.error)
                                };
                                self.log_view.add_msg(err_txt, LogType::Warn)
                            }
                            self.can_clear = true;
                        }
                    },
                    Err(e) => {
                        self.log_view.add_msg(format!("Error reading ECU Errors: {}", e.get_text()), LogType::Error);
                        self.can_clear = false;
                    }
                }
            },
            JsonDiagSessionMsg::ClearErrors => {
                self.can_clear = false;
                match self.server.clear_errors() {
                    Ok(_) => self.log_view.add_msg("Clear ECU Errors OK!", LogType::Info),
                    Err(e) => self.log_view.add_msg(format!("Error clearing ECU Errors: {}", e.get_text()), LogType::Error)
                }
            }
            _ => todo!()
        }
        None
    }

    fn subscription(&self) -> iced::Subscription<Self::msg> {
        Subscription::none()
    }
}