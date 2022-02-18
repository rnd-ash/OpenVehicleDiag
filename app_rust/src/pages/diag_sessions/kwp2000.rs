use std::convert::TryFrom;

use ecu_diagnostics::{kwp2000::{Kwp2000DiagnosticServer, Kwp2000ServerOptions, Kwp2000VoidHandler, SessionType, KWP2000Error, DTCRange}, *, channel::IsoTPSettings, DiagServerResult};
use egui::{Label, Color32, CollapsingHeader};

use crate::{window::{InterfacePage, PageAction}, dyn_hw::DynHardware, pages::status_bar::MainStatusBar};

#[derive(Debug)]
pub enum DiagResponse {
    Ok(String),
    Err(String)
}

pub struct Kwp2000Session {
    dev: DynHardware,
    bar: MainStatusBar,
    layer_opts: IsoTPSettings,
    tp_addr: u32,
    tp_addr_str: String,
    custom_addr: bool,
    require_resp: bool,
    addrs: (u32, u32),
    server: Option<Kwp2000DiagnosticServer>,
    error: Option<String>,
    last_response: Option<DiagResponse>,
    session_mode: SessionType,
    custom_sbytes_str: String,
    custom_sbytes: Vec<u8>
}


impl Kwp2000Session {
    pub fn new(dev: DynHardware, bar: MainStatusBar, layer_opts: IsoTPSettings, addrs: (u32, u32)) -> Self {
        Self {
            dev,
            bar,
            layer_opts,
            addrs,
            server: None,
            tp_addr: 0,
            custom_addr: false,
            require_resp: true,
            error: None,
            tp_addr_str: "0".into(),
            last_response: None,
            session_mode: SessionType::Normal,
            custom_sbytes_str: "10 92".into(),
            custom_sbytes: vec![],
        }
    }

    fn exec_diag_job<X: FnOnce(&mut Kwp2000DiagnosticServer) -> DiagServerResult<String>>(&mut self, name: &str, f: X) -> bool {
        self.last_response = None;
        if let Some(mut server) = self.server.as_mut() {
            match f(&mut server) {
                Ok(r) => {
                    self.last_response = Some(DiagResponse::Ok(format!("{} {}", name, r)));
                    return true;
                },
                Err(e) => {
                    self.last_response = Some(DiagResponse::Err(format!("{} failed!: {}", name, e)));
                    return false;
                }
            }
        }
        return false;
    }

    fn build_diag_ui(&mut self, ui: &mut egui::Ui) -> crate::window::PageAction {
        CollapsingHeader::new("Session control").default_open(false).show(ui,|sub| {
            sub.horizontal_wrapped(|row| {
                if row.button("Default mode").clicked() {
                    if self.exec_diag_job("Enter default session", |server| {
                        kwp2000::set_diagnostic_session_mode(server, SessionType::Normal).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::Normal;
                    }
                }
                if row.button("Reprogramming mode").clicked() {
                    if self.exec_diag_job("Enter reprogramming session", |server| {
                        kwp2000::set_diagnostic_session_mode(server, SessionType::Reprogramming).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::Reprogramming;
                    }
                }
                if row.button("Standby mode").clicked() {
                    if self.exec_diag_job("Enter standby session", |server| {
                        kwp2000::set_diagnostic_session_mode(server, SessionType::Standby).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::Standby;
                    }
                }
                if row.button("Passive mode").clicked() {
                    if self.exec_diag_job("Enter passive session", |server| {
                        kwp2000::set_diagnostic_session_mode(server, SessionType::Passive).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::Passive;
                    }
                }
                if row.button("Extended diagnostics mode").clicked() {
                    if self.exec_diag_job("Enter extended diagnostic session", |server| {
                        kwp2000::set_diagnostic_session_mode(server, SessionType::ExtendedDiagnostics).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::ExtendedDiagnostics;
                    }
                }
            })
        });

        CollapsingHeader::new("ECU Reset").default_open(false).show(ui,|sub| {
            sub.colored_label(Color32::from_rgb(255,0,0), "!! USE WITH CAUTION !!");
            sub.horizontal_wrapped(|row| {
                if row.button("Power on reset").clicked() {
                    if self.exec_diag_job("Power on reset", |server| {
                        kwp2000::execute_reset(server, kwp2000::ResetMode::PowerOnReset).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::Normal;
                    }
                }
                if row.button("Nonvolatile memory reset").clicked() {
                    if self.exec_diag_job("Nonvolatile memory reset", |server| {
                        kwp2000::execute_reset(server, kwp2000::ResetMode::NonVolatileMemoryReset).map(|_| "OK".to_string())
                    }) {
                        self.session_mode = SessionType::Normal;
                    }
                }
                
            });
        });

        CollapsingHeader::new("ECU Identification").default_open(false).show(ui,|sub| {
            sub.horizontal_wrapped(|row| {
                if row.button("Daimler info").clicked() {
                    self.exec_diag_job("Daimler info", |server| {
                        kwp2000::read_daimler_identification(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Daimler/MMC info").clicked() {
                    self.exec_diag_job("Daimler/MMC info", |server| {
                        kwp2000::read_daimler_mmc_identification(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Original VIN").clicked() {
                    self.exec_diag_job("Original VIN", |server| {
                        kwp2000::read_original_vin(server)
                    });
                }
                if row.button("Diag variant code").clicked() {
                    self.exec_diag_job("Diag variant code", |server| {
                        kwp2000::read_diagnostic_variant_code(server).map(|i| format!("{}", i))
                    });
                }
                // System supplier specific $8A-$8F
                if row.button("Current VIN").clicked() {
                    self.exec_diag_job("Current VIN", |server| {
                        kwp2000::read_current_vin(server)
                    });
                }
                if row.button("Calibration identifier").clicked() {
                    self.exec_diag_job("Calibration identifier", |server| {
                        kwp2000::read_calibration_id(server)
                    });
                }
                if row.button("CVN").clicked() {
                    self.exec_diag_job("CVN", |server| {
                        kwp2000::read_cvn(server).map(|i| format!("{:02X?}", i))
                    });
                }

                if row.button("Code fingerprint").clicked() {
                    self.exec_diag_job("Code fingerprint", |server| {
                        kwp2000::read_ecu_code_fingerprint(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Data fingerprint").clicked() {
                    self.exec_diag_job("Data fingerprint", |server| {
                        kwp2000::read_ecu_data_fingerprint(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Code software identification").clicked() {
                    self.exec_diag_job("Code software identification", |server| {
                        kwp2000::read_ecu_code_software_id(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Data software identification").clicked() {
                    self.exec_diag_job("Data software identification", |server| {
                        kwp2000::read_ecu_data_software_id(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Boot software identification").clicked() {
                    self.exec_diag_job("Boot software identification", |server| {
                        kwp2000::read_ecu_boot_software_id(server).map(|i| format!("{:#?}", i))
                    });
                }
                if row.button("Boot fingerprint").clicked() {
                    self.exec_diag_job("Boot fingerprint", |server| {
                        kwp2000::read_ecu_boot_fingerprint(server).map(|i| format!("{:#?}", i))
                    });
                }
            });
        });

        CollapsingHeader::new("Read / Clear diagnostic trouble codes").default_open(false).show(ui,|sub| {
            sub.horizontal_wrapped(|row| {
                if row.button("Identified DTCs (SAE J2012/ISO15031-6)").clicked() {
                    self.exec_diag_job("Identified DTCs", |server| {
                        kwp2000::read_stored_dtcs_iso15031(server, DTCRange::All).map(|i| {
                            let mut res = format!("({} found): \n", i.len());
                            for dtc in &i {
                                res.push_str(&format!("{} - CEL: {}, Status: {:?}. RAW: {:04X}\n", dtc.get_name_as_string(), dtc.mil_on, dtc.status, dtc.raw))
                            }
                            res
                        })
                    });
                }
                if row.button("Supported DTCs (SAE J2012/ISO15031-6)").clicked() {
                    self.exec_diag_job("Supported DTCs", |server| {
                        kwp2000::read_supported_dtcs_iso15031(server, DTCRange::All).map(|i| {
                            let mut res = String::new();
                            for dtc in &i {
                                res.push_str(&format!("{} (RAW: {:04X})\n", dtc.get_name_as_string(), dtc.raw))
                            }
                            res
                        })
                    });
                }
                if row.button("Identified 2 byte hex DTC and status").clicked() {
                    self.exec_diag_job("Identified DTCs", |server| {
                        kwp2000::read_stored_dtcs(server, DTCRange::All).map(|i| {
                            let mut res = format!("({} found): \n", i.len());
                            for dtc in &i {
                                res.push_str(&format!("{} - CEL: {}, Status: {:?}. RAW: {:04X}\n", dtc.get_name_as_string(), dtc.mil_on, dtc.status, dtc.raw))
                            }
                            res
                        })
                    });
                }
                if row.button("Supported 2 byte hex DTC and status").clicked() {
                    self.exec_diag_job("Supported DTCs", |server| {
                        kwp2000::read_supported_dtcs(server, DTCRange::All).map(|i| {
                            let mut res = String::new();
                            for dtc in &i {
                                res.push_str(&format!("{} (RAW: {:04X})\n", dtc.get_name_as_string(), dtc.raw))
                            }
                            res
                        })
                    });
                }
                if row.button("Most recent DTC").clicked() {
                    self.exec_diag_job("Most recent DTC", |server| {
                        Err(DiagError::NotImplemented("Most recent DTC is not implemented yet".into()))
                    });
                }
                if row.button("Clear stored DTCs").clicked() {
                    self.exec_diag_job("Clear DTCs", |server| {
                        kwp2000::clear_dtc(server, kwp2000::ClearDTCRange::AllDTCs).map(|_| "OK".into())
                    });
                }
            });
        });

        CollapsingHeader::new("Read data by identifier").default_open(false).show(ui,|sub| {
            sub.horizontal_wrapped(|row| {
                if row.button("ECU Serial number").clicked() {
                    self.exec_diag_job("ECU Serial number", |server| {
                        kwp2000::read_ecu_serial_number(server).map(|pres| {
                            format!("As ASCII: \"{}\", As BCD: \"{}\"", String::from_utf8_lossy(&pres).to_string(), ecu_diagnostics::helpers::bcd_decode_slice(&pres, None))
                        })
                    });
                }
            });
        });

        let mut t = self.custom_sbytes_str.clone();
        ui.horizontal_wrapped(|row| {
            row.label("Enter custom payload (HEX) EG '10 92':");
            row.text_edit_singleline(&mut t);
            let mut bytes_maybe: Vec<u8> = Vec::new();
            for s in t.split(" ") {
                if let Ok(parsed) = u8::from_str_radix(s, 16) {
                    bytes_maybe.push(parsed);
                } else {
                    bytes_maybe = Vec::new();
                    break;
                }
            }

            if !bytes_maybe.is_empty() {
                if row.button("Send").clicked() {
                    self.exec_diag_job("Custom payload", |server| {
                        server.send_byte_array_with_response(&bytes_maybe).map(|r| format!("Response: {:02X?}", r))
                    });
                }
            }

        });
        self.custom_sbytes_str = t;

        if ui.button("Scrape services (Takes time!)").clicked() {
            /*
            let mut msg = String::new();
            for sid in 0x4000..=0x4FFF as u16 {
                if let Ok (data) = self.server.as_mut().unwrap().send_byte_array_with_response(&[0x22, (sid >> 8) as u8, sid as u8]) {
                    use std::fmt::Write;
                    write!(msg, "{:04X} -> {:02X?}", sid, data);
                }
            }
            */
            let mut supported_sids : Vec<u8> = vec![];
            for sid in 0x00..=0xFF as u8 {
                let supported = match self.server.as_mut().unwrap().send_byte_array_with_response(&[sid]) {
                    Ok(_) => true,
                    Err(e) => {
                        match e {
                            DiagError::ECUError { code, def } => {
                                if code == 0x11 {
                                    false
                                } else {
                                    true
                                }
                            },
                            _ => true
                        }
                    }
                };
                if supported {
                    supported_sids.push(sid);
                }
            }
            let mut msg = format!("Supported SIDs (HEX): {:02X?}. Descriptions:\n", supported_sids);
            for x in supported_sids {
                if x <= 0x0A {
                    let obd_name: &str = match x {
                        0x01 => "OBD ShowCurrentData",
                        0x02 => "OBD ShowFreezeFrameData",
                        0x03 => "OBD ShowStoredDTCs",
                        0x04 => "OBD ClearDTCs",
                        0x05 => "OBD O2TestResultsNonCAN",
                        0x06 => "OBD ComponentTestResultsCAN",
                        0x07 => "OBD ShowPendingDTCs",
                        0x08 => "OBD ControlOnBoardSystem",
                        0x09 => "OBD RequestVehicleInfo",
                        0x0A => "OBD ShowPermanentDTCs",
                        _ => "UNKNOWN OBD SID!"
                    };
                    msg.push_str(&format!("{:02X} - {}\n", x, obd_name))
                } else {
                    match kwp2000::KWP2000Command::from(x) {
                        kwp2000::KWP2000Command::CustomSid(_) => msg.push_str(&format!("{:02X} - UNKNOWN KWP SID!\n", x)),
                        f => msg.push_str(&format!("{:02X} - KWP {:?}\n", x, f))
                    }
                }
            }
            self.last_response = Some(DiagResponse::Ok(msg));
        }


        if let Some(r) = &self.last_response {
            match r {
                DiagResponse::Ok(msg) => {
                    ui.add(Label::new(msg).wrap(true).text_color(Color32::from_rgb(0,255,0)));
                    //ui.colored_label(Color32::from_rgb(0,255,0), msg);
                },
                DiagResponse::Err(e) => {
                    ui.colored_label(Color32::from_rgb(255,0,0), e);
                },
            }
        } else {
            ui.label("No action performed");
        }
        ui.label(format!("Current session type: {}", match self.session_mode {
            SessionType::Normal => "Default",
            SessionType::Reprogramming => "Reprogramming",
            SessionType::Standby => "Standby",
            SessionType::Passive => "Passive",
            SessionType::ExtendedDiagnostics => "Extended diagnostics",
            SessionType::Custom(_) => "Custom",
        }));
        PageAction::None
    }
}

impl InterfacePage for Kwp2000Session {
    fn make_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::epi::Frame<'_>) -> crate::window::PageAction {
        ui.label("KWP2000 session");
        if let Some(kwp2000) = &self.server {
            return self.build_diag_ui(ui);
        } else {
            ui.label("KWP2000 options");
            ui.checkbox(&mut self.require_resp, "Tester present requires response?");
            if ui.checkbox(&mut self.custom_addr, "ADVANCED. Use custom address for TP msg?").clicked() {
                if self.custom_addr {
                    self.tp_addr = 0; // Reset
                    self.tp_addr_str = "0".into();
                }
            }
            let mut custom_addr_parsed = true;
            if self.custom_addr {
                ui.label("Enter custom tester present address (Hex)");
                ui.text_edit_singleline(&mut self.tp_addr_str);
                if let Ok(parsed) = u32::from_str_radix(&self.tp_addr_str, 16) {
                    self.tp_addr = parsed;
                    self.error = None;
                } else {
                    self.error = Some(format!("'{}' is not a valid hex input", self.tp_addr_str));
                    custom_addr_parsed = false;
                }
            }
            if custom_addr_parsed {
                if ui.button("Connect to ECU!").clicked() {
                    self.session_mode = SessionType::Normal;
                    let kwp = Kwp2000ServerOptions {
                        send_id: self.addrs.0,
                        recv_id: self.addrs.1,
                        read_timeout_ms: 10000,
                        write_timeout_ms: 10000,
                        global_tp_id: if self.custom_addr {
                            self.tp_addr
                        } else {
                            0
                        },
                        tester_present_interval_ms: 2000,
                        tester_present_require_response: self.require_resp,
                    };
                    match self.dev.create_iso_tp_channel() {
                        Ok(channel) => {
                            match Kwp2000DiagnosticServer::new_over_iso_tp(kwp, channel, self.layer_opts, Kwp2000VoidHandler{}) {
                                Ok(server) => { self.server = Some(server) }
                                Err(e) => {
                                    self.error = Some(format!("Error starting KWP2000 server: {}", e))
                                }
                            }
                        },
                        Err(e) => {
                            self.error = Some(format!("Error starting KWP2000 server: {}", e))
                        }
                    }
                }
            }
            if let Some(e) = &self.error {
                ui.colored_label(Color32::from_rgb(255,0,0), &e);
            }
        }
        PageAction::None
    }

    fn get_title(&self) -> &'static str {
        "OpenVehicleDiag Diagnostic home"
    }

    fn get_status_bar(&self) -> Option<Box<dyn crate::window::StatusBar>> {
        Some(Box::new(self.bar.clone()))
    }
}