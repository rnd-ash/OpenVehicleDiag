use eframe::epi;
use egui::Color32;

use crate::{dyn_hw::DynHardware, window::{InterfacePage, StatusBar, PageAction}};

use super::{status_bar::{MainStatusBar}, can_tracer::CanTracerPage, diag_home::DiagHome};

pub struct HomePage {
    dev: DynHardware,
    bar: MainStatusBar
}

impl HomePage {
    pub fn new(dev: DynHardware) -> Self {
        Self {
            dev: dev.clone(),
            bar: MainStatusBar::new(dev)
        }
    }
}

fn gen_cap_text(name: &str, cap: bool, ui: &mut egui::Ui) {
    ui.horizontal(|row| {
        row.label(format!("Supports {}?", name));
        match cap {
            true => row.colored_label(Color32::from_rgb(0, 255, 0), "Yes"),
            false => row.label("No")
        }
    });
}

fn gen_info_text(name: &str, data: &Option<String>, ui: &mut egui::Ui) {
    ui.horizontal(|row| {
        row.label(format!("Device {}:", name));
        match data {
            Some(s) => row.label(s),
            None => row.label("N/A")
        }
    });
}

impl InterfacePage for HomePage {
    fn make_ui(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) -> crate::window::PageAction {
        let info = self.dev.get_info();
        let caps = info.capabilities;
        ui.vertical_centered(|col| {
            col.label("Welcome to OpenVehicleDiag!");
            col.add_space(10.0);
            col.label("Device info");
            gen_info_text("vendor", &info.vendor, col);
            gen_info_text("library path", &info.library_location, col);
            gen_info_text("library version", &info.library_version, col);
            gen_info_text("firmware version", &info.device_fw_version, col);
            gen_info_text("API version", &info.api_version, col);
            col.label("Device capabilities");
            gen_cap_text("CAN", caps.can, col);
            gen_cap_text("ISO-TP", caps.iso_tp, col);
            gen_cap_text("ISO 14230", caps.kline_kwp, col);
            gen_cap_text("ISO 9141", caps.kline, col);
            gen_cap_text("SAE J1850", caps.sae_j1850, col);
            gen_cap_text("DOIP", caps.ip, col);
            gen_cap_text("SCI", caps.sci, col);
        });
        ui.label("OVD Functions");
        if ui.button("CAN Tracer").clicked() {
            return PageAction::Add(
                Box::new(
                    CanTracerPage::new(self.dev.clone(), self.bar.clone())
                )
            )
        }
        if ui.button("ECU Diagnostics").clicked() {
            return PageAction::Add(
                Box::new(
                    DiagHome::new(self.dev.clone(), self.bar.clone())
                )
            )
        }
        crate::window::PageAction::None
    }

    fn get_title(&self) -> &'static str {
        "OpenVehicleDiag home"
    }

    fn get_status_bar(&self) -> Option<Box<dyn StatusBar>> {
        Some(Box::new(self.bar.clone()))
    }
}