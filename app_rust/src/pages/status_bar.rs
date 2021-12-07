use std::sync::{Arc, RwLock};

use ecu_diagnostics::hardware::Hardware;
use eframe::epi;
use egui::Color32;

use crate::{dyn_hw::DynHardware, window::{InterfacePage, StatusBar}};

#[derive(Clone)]
pub struct MainStatusBar {
    pub hw: DynHardware,
    last_voltage: Arc<RwLock<Option<f32>>>
}

impl MainStatusBar {
    pub fn new(hw: DynHardware) -> Self {
        if let Some(batt) = hw.clone().read_battery_voltage() {
            // Spin up the reader thread
            let mut c = hw.clone();
            let batt = Arc::new(RwLock::new(Some(batt)));
            let batt_c = batt.clone();
            std::thread::spawn(move || {
                loop {
                    if let Ok(mut sto) = batt_c.write() {
                        *sto = c.read_battery_voltage();
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2000))
                }
            });
            Self {
                hw,
                last_voltage: batt
            }
        } else {
            Self {
                hw,
                last_voltage: Arc::new(RwLock::new(None))
            }
        }
    }
}

impl StatusBar for MainStatusBar {
    fn draw(&mut self, ui: &mut egui::Ui) {
        let b = *self.last_voltage.read().unwrap();
        ui.horizontal(|row| {
            if let Some(batt) = b {
                let colour = match batt {
                    0.0..=10.5 => Color32::from_rgb(255, 0, 0),
                    10.5..=11.5 => Color32::from_rgb(128, 128, 0),
                    _ => Color32::from_rgb(0, 255, 0)
                };
                row.colored_label(colour, format!("Battery voltage: {:.1}V", batt));
            } else {
                row.label("Battery voltage unsupported");
            }
            if self.hw.is_connected() {
                row.colored_label(Color32::from_rgb(0,255,0), "Connected");
            } else {
                row.label("Not connected");
            }
        });
    }
}