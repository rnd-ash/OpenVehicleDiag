use std::{fs::File, io::Read, path::Path, todo};

use iced::{Align, Column, Element, Length, Row};

use crate::{commapi::comm_api::{ComServer, ISO15765Config}, themes::{ButtonType, TextType, TitleSize, button_outlined, picklist, text, title_text}};

use super::{diag_home::{ECUDiagSettings, VehicleECUList}, diag_session::{DiagSession, SessionMsg, SessionType}};

#[derive(Debug, Clone)]
pub enum DiagManualMessage {
    LaunchFileBrowser,
    LoadFile(String),
    PickECU(ECUDiagSettings),
    LaunchKWP,
    LaunchUDS,
    LaunchCustom,
    Back,
    Session(SessionMsg)
}


#[derive(Debug, Clone)]
pub struct DiagManual {
    server: Box<dyn ComServer>,
    car: Option<VehicleECUList>,
    btn_state: iced::button::State,
    pick_state: iced::pick_list::State<ECUDiagSettings>,
    status: String,
    curr_ecu: Option<ECUDiagSettings>,
    uds_btn_state: iced::button::State,
    kwp_btn_state: iced::button::State,
    custom_btn_state: iced::button::State,
    session: Option<DiagSession>
}

impl DiagManual {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            car: None,
            btn_state: Default::default(),
            pick_state: Default::default(),
            status: "".into(),
            curr_ecu: None,
            uds_btn_state: Default::default(),
            kwp_btn_state: Default::default(),
            custom_btn_state: Default::default(),
            session: None,
        }
    }

    pub fn update(&mut self, msg: &DiagManualMessage) -> Option<DiagManualMessage> {
        // If session is active, all calls get re-directed to the active diag session
        if let Some(ref mut session) = self.session {
            if let DiagManualMessage::Session(m) = msg {
                return session.update(m).map(DiagManualMessage::Session)
            }
        }
        match msg {
            DiagManualMessage::Back => {
                //if 
            }
            DiagManualMessage::LaunchFileBrowser => {
                if let nfd::Response::Okay(f_path) = nfd::open_file_dialog(Some("ovdjson"), None).unwrap_or(nfd::Response::Cancel) {
                    let path = f_path.clone();
                    if let Ok(mut file) = File::open(f_path) {
                        let mut str = "".into();
                        file.read_to_string(&mut str);

                        let parse : serde_json::Result<VehicleECUList>  = serde_json::from_str(&str);
                        match parse {
                            Ok(car) => {
                                self.curr_ecu = None;
                                self.car = Some(car)
                            },
                            Err(e) => {
                                self.status = format!("Error processing {}: {}", path, e)
                            }
                        }
                    } else {
                        self.status = format!("Error loading save file")
                    }
                }
            }
            DiagManualMessage::PickECU(e) => {
                 self.curr_ecu = Some(e.clone())
            }
            DiagManualMessage::LaunchKWP => {
                self.launch_diag_session(SessionType::KWP)
            }
            DiagManualMessage::LaunchUDS => {
                self.launch_diag_session(SessionType::UDS)
            }
            DiagManualMessage::LaunchCustom => {
                self.launch_diag_session(SessionType::Custom)
            }
            _ => {}
        }
        None
    }

    pub fn launch_diag_session(&mut self, session_type: SessionType) {
        if self.session.is_some() {
            self.status = "Error. Diagnostic session already in progress??".into(); // How did this happen??
            return
        }

        if let Some(ecu) = &self.curr_ecu {

            let cfg = ISO15765Config {
                send_id: ecu.send_id,
                recv_id: ecu.flow_control_id,
                block_size: ecu.block_size,
                sep_time: ecu.sep_time_ms,
            };
            self.session = Some(DiagSession::new(&session_type, self.server.clone(), cfg));
        } else {
            self.status = "Error. No ECU selected?".into(); // How did this happen??
        }
    }

    pub fn view(&mut self) -> Element<DiagManualMessage> {
        if let Some(ref mut session) = self.session {
            return session.view().map(DiagManualMessage::Session)
        }
        let mut view = Column::new().padding(20).spacing(20).align_items(Align::Center).width(Length::Fill)
        .push(title_text("Load a save file to get started", TitleSize::P3));

        view = view.push(button_outlined(&mut self.btn_state,"Load save file", ButtonType::Success).on_press(DiagManualMessage::LaunchFileBrowser));

        if let Some(car) = &self.car {
            view = view.push(text(format!("Loaded car: {} {} ({})", car.vehicle_brand, car.vehicle_name, car.vehicle_year).as_str(), TextType::Normal));

            view = view.push(picklist(&mut self.pick_state, car.ecu_list.clone(), self.curr_ecu.clone(), DiagManualMessage::PickECU));

            if let Some(ecu) = &self.curr_ecu {

                let kwp_text = if ecu.kwp_support { "Launch KWP2000 session" } else { "KWP2000 not supported" };
                let mut kwp_btn = button_outlined(&mut self.kwp_btn_state, kwp_text, ButtonType::Primary).width(Length::Units(250));
                if ecu.kwp_support {
                    kwp_btn = kwp_btn.on_press(DiagManualMessage::LaunchKWP);
                }

                let uds_text = if ecu.uds_support { "Launch UDS session" } else { "UDS not supported" };
                let mut uds_btn = button_outlined(&mut self.uds_btn_state, uds_text, ButtonType::Primary).width(Length::Units(250));
                if ecu.uds_support {
                    uds_btn = uds_btn.on_press(DiagManualMessage::LaunchUDS);
                }

                let custom_btn = button_outlined(&mut self.custom_btn_state, "Launch Custom session", ButtonType::Warning).on_press(DiagManualMessage::LaunchCustom).width(Length::Units(250));

                view = view.push(
                    Row::new().spacing(8).push(kwp_btn)
                    .push(uds_btn)
                    .push(custom_btn)
                )

            }

        }

        view = view.push(title_text("Or specify manual ISO-TP Settings", TitleSize::P3));

        view = view.push(text(&self.status, TextType::Danger));

        view.into()
    }
}