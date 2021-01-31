use common::schema::{OvdECU, variant::{ECUVariantDefinition, ECUVariantPattern}};
use iced::{Column, Subscription};

use crate::{commapi::{comm_api::{ComServer, ISO15765Config}, protocols::{DiagProtocol, DiagServer, ProtocolServer, kwp2000::{KWP2000ECU, Service}}}, themes::text, windows::diag_manual::DiagManualMessage};

use super::{DiagMessageTrait, SessionError, SessionMsg, SessionResult, SessionTrait};

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
    server: DiagServer,
    ecu: ISO15765Config,
    ecu_text: (String, String),
    ecu_data: ECUVariantDefinition,
    pattern: ECUVariantPattern
}

impl JsonDiagSession {
    pub fn new(comm_server: Box<dyn ComServer>, ecu: ISO15765Config, ecu_data: OvdECU) -> SessionResult<Self> {
        match DiagServer::new(comm_server, &ecu, DiagProtocol::KWP2000) {
            Ok(mut server) => {
                let res = server.run_cmd(Service::ReadECUID, &[0x87], 500)?;
                println!("{:02X?}", res);

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
                        pattern
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
        Column::new()
            .push(text(format!("ECU: {} ({}). DiagVersion: {}, Vendor: {}", self.ecu_text.0, self.ecu_text.1, self.ecu_data.name, self.pattern.vendor).as_str(), crate::themes::TextType::Normal))
            .into()
    }

    fn update(&mut self, msg: &Self::msg) -> Option<Self::msg> {
        todo!()
    }

    fn subscription(&self) -> iced::Subscription<Self::msg> {
        Subscription::none()
    }
}