use std::ops::Index;

use crate::{window::{InterfacePage, PageAction}, dyn_hw::DynHardware, pages::home::HomePage};
use ecu_diagnostics::{hardware::{passthru::PassthruScanner, HardwareScanner, HardwareInfo, HardwareResult}, HardwareError};
#[cfg(unix)]
use ecu_diagnostics::hardware::socketcan::SocketCanScanner;
use eframe::epi;
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
    old_api: CommunicationApi,
    selected: String,
    old_selected: String,
    launch_err: Option<String>
}

impl Launcher {
    pub fn new() -> Self {
        Self{
            api:CommunicationApi::Passthru,
            old_api: CommunicationApi::Passthru,
            selected: "".into(),
            old_selected: "".into(),
            launch_err: None
        }
    }
}

impl Launcher {
    pub fn open_device(api: &CommunicationApi, idx: usize) -> HardwareResult<DynHardware> {
        match api {
            CommunicationApi::Passthru => PassthruScanner::new().open_device_by_index(idx).map(|d| DynHardware::new_from_passthru(d)),
            #[cfg(unix)]
            CommunicationApi::SocketCAN => SocketCanScanner::new().open_device_by_index(idx).map(|d| DynHardware::new_from_socketcan(d)),
            #[cfg(not(unix))]
            CommunicationApi::SocketCAN => Err(HardwareError::DeviceNotFound),
            CommunicationApi::Dpdu => Err(HardwareError::DeviceNotFound)
        }
    }
    pub fn scan_devices(api: CommunicationApi) -> ScanResult {
        match api {
            CommunicationApi::Passthru => {
                Ok(PassthruScanner::new().list_devices().iter().map(|d| format!("{} by {}", d.name, d.vendor.clone().unwrap())).collect())
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

    fn make_ui(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) -> crate::window::PageAction {
        ui.label("Welcome to OpenVehicleDiag!");
        ui.label("Please select a device to get started");
        ui.vertical_centered(|center| {
            center.horizontal(|row| {
                row.radio_value(&mut self.old_api, CommunicationApi::Passthru, "Passthru (SAE J2534)");
                row.radio_value(&mut self.old_api, CommunicationApi::SocketCAN, "SocketCAN");
                row.radio_value(&mut self.old_api, CommunicationApi::Dpdu, "D-PDU (ISO 22900-2)");
            });
        });
        if self.api != self.old_api {
            self.api = self.old_api;
            self.launch_err = None;
        }

        match Launcher::scan_devices(self.api) {
            Ok(list) => {
                if list.len() > 0 {
                    egui::ComboBox::from_label("Select device")
                        .width(400.0)
                        .selected_text(&self.selected)
                        .show_ui(ui, |cb_ui| {
                            for x in &list {
                                cb_ui.selectable_value(&mut self.old_selected, x.clone(), x);
                            }
                        });
                        if self.selected != self.old_selected {
                            self.selected = self.old_selected.clone();
                            self.launch_err = None;
                        }
                    if !self.selected.is_empty() {
                        if ui.button("Launch OVD!").clicked() {
                            match Launcher::open_device(&self.api, list.iter().position(|x| x == &self.selected).unwrap_or(99)) {
                                Ok(dev) => {
                                    return PageAction::Overwrite(Box::new(HomePage::new(dev)))
                                },
                                Err(e) => {
                                    println!("Error trying to start {} using {:?}: {}", self.selected, self.api, e);
                                    self.launch_err = Some(format!("Could not launch OVD with specified adapter: {}", e));
                                }
                            }

                        }
                    }
                } else {
                    self.launch_err = Some("No devices found".into());
                }
            },
            Err(e) => {
                self.launch_err = Some(format!("Error: {}", e));
            },
        }
        if let Some(e) = &self.launch_err {
            ui.colored_label(Color32::from_rgb(255,0,0), e);
        }
        crate::window::PageAction::None
    }

    fn get_title(&self) -> &'static str {
        "OpenVehicleDiag launcher (EGUI Edition)"
    }

    fn get_status_bar(&self) -> Option<Box<dyn crate::window::StatusBar>> {
        None
    }
}