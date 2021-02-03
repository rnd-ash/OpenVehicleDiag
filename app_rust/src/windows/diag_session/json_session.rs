use std::cmp::min;

use commapi::protocols;
use common::schema::{OvdECU, diag::service::{ParamDecodeError, Service}, variant::{ECUVariantDefinition, ECUVariantPattern}};
use iced::{Align, Column, Container, Length, Row, Subscription};
use protocols::{ECUCommand, Selectable};

use crate::{commapi::{self, comm_api::{ComServer, ISO15765Config}, protocols::{DiagProtocol, DiagServer, ProtocolServer, kwp2000::{KWP2000ECU}}}, themes::{ButtonType, button_outlined, picklist, text, title_text}, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionError, SessionMsg, SessionResult, SessionTrait, log_view::{LogType, LogView}};

type DiagService = commapi::protocols::kwp2000::Service;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonDiagSessionMsg {
    ReadErrors,
    ClearErrors,
    RunService,
    SelectService(ServiceWrapper),
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
    read_only_functions: Vec<ServiceWrapper>,
    execute_service: iced::button::State,
    service_list: iced::pick_list::State<ServiceWrapper>,
    selected_service: Option<ServiceWrapper>,

    read_errors: iced::button::State
}

impl JsonDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config, ecu_data: OvdECU) -> SessionResult<Self> {
        match DiagServer::new(comm_server, &ecu, DiagProtocol::KWP2000) {
            Ok(mut server) => {
                let res = server.run_cmd(DiagService::ReadECUID.get_byte(), &[0x87], 500)?;
                let variant = (res[3] as u32) << 8 | (res[4] as u32);
                let ecu_variant = ecu_data.variants.into_iter().find(|x| {
                    x.clone().patterns.into_iter().any(|p| p.vendor_id == variant)
                });


                if let Some(v) = ecu_variant {
                    let pattern = v.clone().patterns.into_iter().find(|x| x.vendor_id == variant).unwrap();
                    println!("ECU Variant: {} (Vendor: {})", v.name, pattern.vendor);

                    let read_only_functions: Vec<ServiceWrapper> =  v.services
                        .iter()
                        .filter(|x| x.input_params.is_empty() && !x.output_params.is_empty()).cloned()
                        .map(|service| ServiceWrapper{service})
                        .collect();


                    Ok(Self {
                        ecu,
                        ecu_text: (ecu_data.name, ecu_data.description),
                        server,
                        ecu_data: v,
                        pattern,
                        log_view: LogView::new(),
                        read_errors: Default::default(),
                        clear_errors: Default::default(),
                        execute_service: Default::default(),
                        service_list: Default::default(),
                        selected_service: None,
                        // Read only functions that don't require user input
                        read_only_functions,
                        can_clear: false
                    })
                } else {
                    Err(SessionError::Other(format!("Could not locate ECU variant in JSON - Its variant: {}", variant)))
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


        if !self.read_only_functions.is_empty() {
            btn_view = btn_view.push(
                picklist(&mut self.service_list, &self.read_only_functions, self.selected_service.clone(), JsonDiagSessionMsg::SelectService)
            );

            if self.selected_service.is_some() {
                btn_view = btn_view.push(text(self.selected_service.clone().unwrap().service.description.as_str(), crate::themes::TextType::Normal));
                btn_view = btn_view.push(button_outlined(&mut self.execute_service, "Execute", ButtonType::Warning).on_press(JsonDiagSessionMsg::RunService))
            }
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
        //self.log_view.clear_logs();
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
                                let desc = self.ecu_data.errors.clone().into_iter().find(|x| x.error_name.ends_with(e.error.as_str()));
                                let err_txt = match desc {
                                    Some(d) =>  format!("{} - {}", e.error, d.description),
                                    None => format!("{} - Unknown description", e.error)
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

            JsonDiagSessionMsg::RunService => {
                if let Some(service) = &self.selected_service {
                    let com_dat = service.get_payload();
                    match self.server.run_cmd(com_dat.0, &com_dat.1, 1000) {
                        Ok(res) => {
                            let mut resp_vec = vec![com_dat.0];
                            resp_vec.extend_from_slice(&res);
                            let resp = service.get_value_string(&resp_vec);
                            self.log_view.add_log(
                                format!("Request {} ({:02X?})", service.to_string(), &service.service.payload), 
                                format!("Response: {}", resp), LogType::Info
                            );
                        },

                        Err(e) => self.log_view.add_msg(format!("Error executing {}: {}", service.to_string(), e.get_text()), LogType::Error)
                    }
                }
            }
            JsonDiagSessionMsg::SelectService(s) => {
                self.selected_service = Some(s.clone())
            }
            _ => todo!()
        }
        None
    }

    fn subscription(&self) -> iced::Subscription<Self::msg> {
        Subscription::none()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceWrapper {
    service: Service
}

impl Eq for ServiceWrapper {

}

impl ToString for ServiceWrapper {
    fn to_string(&self) -> String {
        self.service.name.clone()
    }
}

impl ServiceWrapper {
    fn get_payload(&self) -> (u8, Vec<u8>) {
        (
            self.service.payload[0],
            Vec::from(&self.service.payload[1..])
        )
    }

    fn get_value_string(&self, input: &[u8]) -> String {
        let mut s: String = "".into();
        for p in &self.service.output_params {
            match p.decode_value_to_string(input) {
                Ok(res) => s.push_str(format!("{} - {}\n", p.name, res).as_str()),
                Err(err) => s.push_str(format!("ERROR decoding {} - {:?}\n", p.name, err).as_str())
            }
        }
        s.remove(s.len()-1); // Remove trailing \n if present
        s
    }

}

