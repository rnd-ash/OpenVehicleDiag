use crate::{dyn_hw::DynHardware, window::InterfacePage};

pub struct StatusBar {
    pub hw: DynHardware,
    last_voltage: Option<f32>
}

impl StatusBar {
    pub fn new(hw: DynHardware) -> Self {
        let mut s  = Self {
            hw,
            last_voltage: None
        };
        s.last_voltage = s.hw.read_battery_voltage();
        s
    }
}

impl InterfacePage for StatusBar {
    fn make_ui(&mut self, ui: &mut egui::Ui) -> crate::window::PageAction {
        ui.horizontal(|row| {
            if let Some(voltage) = self.last_voltage {
                row.label("Battery voltage: ");
                row.label(format!("{}V", voltage));
            }
        });
        crate::window::PageAction::None
    }

    fn get_title(&self) -> &'static str {
        ""
    }

    fn show_status_bar(&self) -> bool {
        true
    }
}