use crate::window::InterfacePage;
use ecu_diagnostics::{hardware::{passthru::PassthruScanner, HardwareScanner, HardwareInfo}};
#[cfg(unix)]
use ecu_diagnostics::hardware::socketcan::SocketCanScanner;
use egui::{Widget, Color32};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone)]
pub enum CommunicationApi {
    Passthru,
    SocketCAN,
    Dpdu
}


type ScanResult = std::result::Result<Vec<String>, String>;

pub struct Launcher {
    api: CommunicationApi,
    selected: String
}

impl Launcher {
    pub fn new() -> Self {
        Self{
            api:CommunicationApi::Passthru,
            selected: "".into()
        }
    }
}

impl Launcher {
    pub fn scan_devices(api: CommunicationApi) -> ScanResult {
        match api {
            CommunicationApi::Passthru => {
                Ok(PassthruScanner::new().list_devices().iter().map(|d| format!("{} by {}", d.name, d.vendor)).collect())
            },
            CommunicationApi::SocketCAN => {
                #[cfg(unix)]
                 {
                    Ok(SocketCanScanner::new().list_devices().iter().map(|d| format!("{}", d.name)).collect())
                 }
                 #[cfg(not(unix))]
                 {
                     Err("SocketCAN is not supported on this OS!".into())
                 }
            },
            CommunicationApi::Dpdu => {
                Err("This feature is in development, check back soon!".into())
            },
        }
    }
}

impl InterfacePage for Launcher {

    fn make_ui(&mut self, ui: &mut egui::Ui) -> crate::window::PageAction {
        ui.vertical_centered(|center| {
            center.horizontal(|row| {
                row.radio_value(&mut self.api, CommunicationApi::Passthru, "Passthru (SAE J2534)");
                row.radio_value(&mut self.api, CommunicationApi::SocketCAN, "SocketCAN");
                row.radio_value(&mut self.api, CommunicationApi::Dpdu, "D-PDU (ISO 22900-2)");
            });
        });

        match Launcher::scan_devices(self.api) {
            Ok(list) => {
                if list.len() > 0 {
                    egui::ComboBox::from_label("Select device")
                        .width(400.0)
                        .selected_text(&self.selected)
                        .show_ui(ui, |cb_ui| {
                            for x in list {
                                cb_ui.selectable_value(&mut self.selected, x.clone(), x);
                            }
                        });
                } else {
                    ui.colored_label(Color32::from_rgb(255,0,0), format!("No devices found"));
                }
            },
            Err(e) => {
                ui.colored_label(Color32::from_rgb(255,0,0), format!("Error: {}", e));
            },
        }

        
        crate::window::PageAction::None
    }

    fn get_title(&self) -> &'static str {
        "OpenVehicleDiag launcher (EGUI Edition)"
    }

    fn show_status_bar(&self) -> bool {
        false
    }
}