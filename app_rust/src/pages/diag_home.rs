use egui::Label;

use crate::{window::{InterfacePage, PageAction}, dyn_hw::DynHardware};

use super::{status_bar::MainStatusBar, diag_sessions::kwp2000::Kwp2000Session};

const ISO_TP_BS: &[u32] = &[
    0,
    8,
    16,
    32
];

const ISO_TP_ST_MIN: &[u32] = &[
    0,
    10,
    20
];

pub struct DiagHome {
    dev: DynHardware,
    bar: MainStatusBar,
    send_addr_string: String,
    recv_addr_string: String,
    pad_frames: bool,
    block_size: u32,
    st_min: u32,
}


impl DiagHome {
    pub fn new(dev: DynHardware, bar: MainStatusBar) -> Self {
        Self {
            dev,
            bar,
            send_addr_string: "07E0".into(),
            recv_addr_string: "07E8".into(),
            block_size: 8,
            st_min: 20,
            pad_frames: true
        }
    }
}

impl InterfacePage for DiagHome {
    fn make_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::epi::Frame<'_>) -> crate::window::PageAction {
        ui.label("Diagnostic home");
        ui.add_space(20.0);
        let mut recv = self.recv_addr_string.clone();
        let mut send = self.send_addr_string.clone();
        ui.label("ISO-TP options");
        ui.horizontal(|row| {
            row.label("Enter Send address (Hex)");
            row.text_edit_singleline(&mut send);
            row.add_space(30.0);
            row.label("Enter Receive address (Hex)");
            row.text_edit_singleline(&mut recv);
        });
        self.send_addr_string = send;
        self.recv_addr_string = recv;
        ui.horizontal(|row| {
            row.checkbox(&mut self.pad_frames, "Pad ISO-TP frames?");
            row.add_space(30.0);
            egui::ComboBox::from_label("Seperation time (ms)")
                .width(100.0)
                .selected_text(&mut self.st_min)
                .show_ui(row, |combo| {
                    for x in ISO_TP_ST_MIN {
                        combo.selectable_value(&mut self.st_min, *x, format!("{} ms", x));
                    }
                });
            row.add_space(30.0);
            egui::ComboBox::from_label("Block size")
                .width(100.0)
                .selected_text(&mut self.block_size)
                .show_ui(row, |combo| {
                    for x in ISO_TP_BS {
                        combo.selectable_value(&mut self.st_min, *x, format!("{}", x));
                    }
                });
        });
        let s_addr = u32::from_str_radix(&self.send_addr_string, 16).ok();
            let r_addr = u32::from_str_radix(&self.recv_addr_string, 16).ok();
            if let Some(sa) = s_addr {
                if let Some(ra) = r_addr {
                    let iso_tp_opts = ecu_diagnostics::channel::IsoTPSettings {
                        block_size: self.block_size as u8,
                        st_min: self.st_min as u8,
                        extended_addressing: false,
                        pad_frame: self.pad_frames,
                        can_speed: 500_000,
                        can_use_ext_addr: false,
                    };
                    let mut next_page: Option<Box<dyn InterfacePage>> = None;
                    ui.horizontal(|row| {
                        if row.button("Launch KWP2000 session").clicked() {
                            next_page = Some(
                                Box::new(Kwp2000Session::new(self.dev.clone(), self.bar.clone(), iso_tp_opts, (sa, ra)))
                            );
                        }
                        if row.button("Launch UDS session").clicked() {
                        }
                        if row.button("Launch custom ISO-TP session").clicked() {
                        }
                    });

                    if let Some(p) = next_page {
                        return PageAction::Add(p)
                    }
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