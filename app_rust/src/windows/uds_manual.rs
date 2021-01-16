use std::{collections::vec_deque, default};

use crate::{commapi::{comm_api::{Capability, ComServer, ISO15765Config}, protocols::{ProtocolServer, kwp2000::{self, KWP2000ECU}, uds::{UDSCommand, UDSRequest}}}, themes::button_coloured};
use iced::{Align, Column, Element, Length, Row, Rule, Space, Text, TextInput, button};
use crate::windows::window::WindowMessage;
use crate::themes::{title_text, text, TextType, button_outlined, ButtonType, TitleSize, picklist};
use crate::commapi::protocols::kwp2000::*;
use super::uds_scanner::ECUISOTPSettings;

#[derive(Debug, Clone)]
pub enum UDSManualMessage {
    SelectECU(String),
    ConnectECU,
    ConnectCustomECU,
    DisconnectECU,
    SendTextInput(String),
    FCTextInput(String),
    BSTextInput(String),
    SepTextInput(String),
    ReadErrors,
    ReadECUID
}

#[derive(Debug, Clone)]
struct CommDetais {
    pub req: String,
    pub res: String
}

#[derive(Debug, Clone)]
pub struct UDSManual {
    server: Box<dyn ComServer>,
    data: Vec<ECUISOTPSettings>,
    curr_ecu_settings: ECUISOTPSettings,
    ecu_names: Vec<String>,
    curr_ecu: String,
    pick_state: iced::pick_list::State<String>,
    in_session: bool,
    state: iced::button::State,
    state2: iced::button::State,
    state3: iced::button::State,
    show_custom_ecu_ui: bool,
    logs: Vec<CommDetais>,
    diag_server: Option<KWP2000ECU>,
    send_text_input: iced::text_input::State,
    fc_text_input: iced::text_input::State,
    sep_text_input: iced::text_input::State,
    bs_text_input: iced::text_input::State,
    scroll_state: iced:: scrollable::State,

    textinput_strings: Vec<String>
}

impl UDSManual {
    pub(crate) fn new(car_data: Vec<ECUISOTPSettings>, server: Box<dyn ComServer>) -> Self {
        let mut names: Vec<String> = car_data.iter().map(|ecu| format!("{} (ID: 0x{:04X})", ecu.name, ecu.send_id)).collect();
        names.push("Custom".into());
        let mut ret = Self {
            server,
            curr_ecu: "Select ECU".into(),
            data: car_data,
            ecu_names: names,
            pick_state: iced::pick_list::State::default(),
            in_session: false,
            curr_ecu_settings: ECUISOTPSettings { name: "".into(), send_id: 0, flow_control_id: 0, block_size: 0, sep_time_ms: 0, supports_uds: None },
            state: iced::button::State::default(),
            state2: iced::button::State::default(),
            state3: iced::button::State::default(),
            show_custom_ecu_ui: false,
            textinput_strings: vec![
                "".into(),
                "".into(),
                "".into(),
                "".into(),
                "".into()
            ],
            send_text_input: iced::text_input::State::default(),
            fc_text_input: iced::text_input::State::default(),
            sep_text_input: iced::text_input::State::default(),
            bs_text_input: iced::text_input::State::default(),
            logs: Vec::new(),
            diag_server: None,
            scroll_state: iced:: scrollable::State::default(),
            
        };
        println!("Manual mode launching");
        // To guarantee everything works as it should, home screen should have NO interfaces open
        if let Err(e) = ret.server.close_can_interface() {
            eprintln!("ERROR closing CAN Interface {}", e)
        }
        if let Err(e) = ret.server.close_iso15765_interface() {
            eprintln!("ERROR closing ISO-TP Interface {}", e)
        }
        ret
    }

    pub fn update(&mut self, msg: &UDSManualMessage) -> Option<UDSManualMessage> {
        match msg {
            UDSManualMessage::SelectECU(ecu) => {
                self.curr_ecu = ecu.clone();
                if ecu == "Custom" {
                    self.show_custom_ecu_ui = true;
                } else {
                    self.show_custom_ecu_ui = false;
                    for e in self.data.iter() {
                        if ecu.contains(format!("{:04X}", e.send_id).as_str()) {
                            self.curr_ecu_settings = e.clone();
                            return Some(UDSManualMessage::ConnectECU);
                        }
                    }
                }
            },
            UDSManualMessage::ConnectECU => {
                self.logs.clear();
                if self.diag_server.is_none() {
                    let cfg = ISO15765Config {
                        send_id: self.curr_ecu_settings.send_id,
                        recv_id: self.curr_ecu_settings.flow_control_id,
                        block_size: self.curr_ecu_settings.block_size,
                        sep_time: self.curr_ecu_settings.sep_time_ms,

                    };

                    if let Ok(server) = KWP2000ECU::start_diag_session(self.server.clone(), &cfg) {
                        self.in_session = true;
                        self.diag_server = Some(server);
                    }
                }
            },
        
            UDSManualMessage::ConnectCustomECU => {
                let send_id = match u32::from_str_radix(self.textinput_strings[0].as_ref(), 16) {
                    Ok(x) => x,
                    Err(_) => return None
                };
                let flow_control_id = match u32::from_str_radix(self.textinput_strings[1].as_ref(), 16) {
                    Ok(x) => x,
                    Err(_) => return None
                };
                let sep_time_ms = match u32::from_str_radix(self.textinput_strings[2].as_ref(), 16) {
                    Ok(x) => x,
                    Err(_) => return None
                };
                let block_size = match u32::from_str_radix(self.textinput_strings[3].as_ref(), 16) {
                    Ok(x) => x,
                    Err(_) => return None
                };
                self.curr_ecu_settings = ECUISOTPSettings {
                    name: "Custom ECU".into(),
                    send_id,
                    flow_control_id,
                    block_size,
                    sep_time_ms,
                    supports_uds: None,

                };
                return Some(UDSManualMessage::ConnectECU);

            }
        
            UDSManualMessage::DisconnectECU => {
                if let Some(ref mut s) = self.diag_server {
                    s.exit_diag_session();
                    self.in_session = false;
                }
                self.diag_server = None;
            }
        
            
            UDSManualMessage::SendTextInput(s) => {
                if s.len() == 0 {
                    self.textinput_strings[0] = s.to_string();
                } else if s.len() <= 4 {
                    if let Ok(_) = i32::from_str_radix(s, 16) {
                        self.textinput_strings[0] = s.to_string();
                    }
                }
            },

            UDSManualMessage::FCTextInput(s) => {
                if s.len() == 0 {
                    self.textinput_strings[1] = s.to_string();
                } else if s.len() <= 4 {
                    if let Ok(_) = i32::from_str_radix(s, 16) {
                        self.textinput_strings[1] = s.to_string();
                    }
                }
            },

            UDSManualMessage::SepTextInput(s) => {
                if s.len() == 0 {
                    self.textinput_strings[2] = s.to_string();
                } else if let Ok(_) = i32::from_str_radix(s, 10) {
                    self.textinput_strings[2] = s.to_string();
                }
            },

            UDSManualMessage::BSTextInput(s) => {
                if s.len() == 0 {
                    self.textinput_strings[3] = s.to_string();
                } else if let Ok(_) = i32::from_str_radix(s, 10) {
                    self.textinput_strings[3] = s.to_string();
                }
            },

            UDSManualMessage::ReadErrors => {
                if let Some(ref mut s) = self.diag_server {
                    match s.read_errors() {
                        Ok(v) => {
                            let mut s: String = "".into();
                            for e in &v {
                                s.push_str(format!("{}\n", e).as_str());
                            }
                            self.logs.push(CommDetais {
                                req: "Send: READ_ECU_ERRORS".into(),
                                res: if v.len() == 0 { "Resp: No Errors".into() } else { format!("Resp: Errors:\n{}", s) }
                            });
                        },
                        Err(e) => {
                            self.logs.push(CommDetais {
                                req: "Send: READ_ECU_ERRORS".into(),
                                res: format!("Err: {:?}", e)
                            });
                        }
                    }
                }
            },
            UDSManualMessage::ReadECUID => {
                if let Some(ref mut s) = self.diag_server {
                    match s.get_ecu_info_data() {
                        Ok(v) => {
                            self.logs.push(CommDetais {
                                req: "Send: READ_ECU_ID".into(),
                                res: format!("Resp: {:#?}", v)
                            });
                        },
                        Err(e) => {
                            self.logs.push(CommDetais {
                                req: "Send: READ_ECU_ID".into(),
                                res: format!("Err: {:?}", e)
                            });
                        }
                    }
                }
            }

            _ => {},
        }
        None
    }

    fn log_run_uds_cmd(&mut self, cmd: UDSRequest) {
        let cfg = ISO15765Config {
            send_id: self.curr_ecu_settings.send_id,
            recv_id: self.curr_ecu_settings.flow_control_id,
            block_size: self.curr_ecu_settings.block_size,
            sep_time: self.curr_ecu_settings.sep_time_ms,
        };
        match cmd.run_cmd_can(&mut self.server, &cfg) {
            Ok(res) => {
                self.logs.push(CommDetais {
                    req: format!("Send: {:02X?}", cmd.to_byte_array()),
                    res: format!("Recv: {:?}", res),

                })
            },
            Err(e) => {
                self.logs.push(CommDetais {
                    req: format!("Send: {:02X?}", cmd.to_byte_array()),
                    res: format!("COMM ERROR: {:?}", e),

                })
            }
        }
    }

    pub fn view(&mut self) -> Element<UDSManualMessage> {
        let mut c = Column::new()
        .spacing(10)
        .padding(10);

        if self.in_session {
            let support = match self.curr_ecu_settings.supports_uds {
                None => "Unknown",
                Some(b) => if b {"Yes"} else {"No"}
            };
            c = c.push(title_text(format!("Connect to {} (0x{:04X}) - UDS Support: {}", 
                                    self.curr_ecu_settings.name, 
                                    self.curr_ecu_settings.send_id,
                                    support
                                ).as_str(), TitleSize::P3))
                .push(button_coloured(&mut self.state, "Disconnect from ECU", ButtonType::Danger).on_press(UDSManualMessage::DisconnectECU)
            );

            let mut comm_view = Column::new(); // Communications overview
            comm_view = comm_view.push(button_outlined(&mut self.state2, "Read ECU errors", ButtonType::Secondary).on_press(UDSManualMessage::ReadErrors))
                .push(button_outlined(&mut self.state3, "Read ECU ID", ButtonType::Secondary).on_press(UDSManualMessage::ReadECUID));



            let mut log_scroll = iced::scrollable::Scrollable::new(&mut self.scroll_state);
            let mut log_view = Column::new(); // Log view
            for log in self.logs.iter() {
                log_view = log_view.push(text(format!("Req: {}", log.req).as_str(), TextType::Normal));
                log_view = log_view.push(text(format!("Res: {}", log.res).as_str(), TextType::Normal));
                log_view = log_view.push(Space::with_height(Length::Units(5)));
            }
            log_scroll = log_scroll.push(log_view);
            c = c.push(Row::new()
            .push(comm_view.width(Length::FillPortion(1)))
            .push(log_scroll.width(Length::FillPortion(1))));
        } else {
            c = c
            .push(title_text("UDS Manual", TitleSize::P2))
            .push(picklist(&mut self.pick_state, &self.ecu_names, Some(self.curr_ecu.clone()), UDSManualMessage::SelectECU));
            if self.show_custom_ecu_ui {
                c = c.push(title_text("Enter ISO-TP details about the custom ECU you wish to connect to", TitleSize::P4))
                    .push(text("Send ID - Hex ID of the Diagnostic sender", TextType::Normal))
                    .push(text("FC ID - Hex ID of the Diagnostic receiver", TextType::Normal))
                    .push(text("Sep time - Separation time in ms between packets", TextType::Normal))
                    .push(text("Block size - Block size for ISO-TP", TextType::Normal));

                c = c.push(Row::new()
                    .padding(10)
                    .spacing(10)
                    .push(TextInput::new(&mut self.send_text_input, "Hex Send ID", &self.textinput_strings[0], UDSManualMessage::SendTextInput).width(Length::Units(150)))
                    .push(TextInput::new(&mut self.fc_text_input, "Hex Flow control ID", &self.textinput_strings[1], UDSManualMessage::FCTextInput).width(Length::Units(150)))
                    .push(TextInput::new(&mut self.sep_text_input, "Sep time (ms)", &self.textinput_strings[2], UDSManualMessage::SepTextInput).width(Length::Units(150)))
                    .push(TextInput::new(&mut self.bs_text_input, "Block size", &self.textinput_strings[3], UDSManualMessage::BSTextInput).width(Length::Units(150)))
                );
                if self.textinput_strings[0] != "" && self.textinput_strings[1] != "" && self.textinput_strings[2] != "" && self.textinput_strings[3] != "" {
                    c = c.push(button_coloured(&mut self.state, "Connect to custom ECU", ButtonType::Primary).on_press(UDSManualMessage::ConnectCustomECU))
                }
            }
        }
        return c.into();
    }
}