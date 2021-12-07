use ecu_diagnostics::{kwp2000::{Kwp2000DiagnosticServer, Kwp2000ServerOptions, Kwp2000VoidHandler, SessionType}, *, channel::IsoTPSettings, DiagServerResult};
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


        if let Some(r) = &self.last_response {
            match r {
                DiagResponse::Ok(msg) => {
                    ui.colored_label(Color32::from_rgb(0,255,0), msg);
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
            if self.custom_addr {
                ui.label("Enter custom tester present address (Hex)");
                ui.text_edit_singleline(&mut self.tp_addr_str);
                if let Ok(parsed) = u32::from_str_radix(&self.tp_addr_str, 16) {
                    self.tp_addr = parsed;
                    self.error = None;
                } else {
                    self.error = Some(format!("'{}' is not a valid hex input", self.tp_addr_str));
                }
            }

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