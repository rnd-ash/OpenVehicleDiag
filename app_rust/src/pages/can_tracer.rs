use std::{borrow::BorrowMut, collections::HashMap, thread::JoinHandle, sync::Arc, time::Instant};

use ecu_diagnostics::{channel::{CanChannel, ChannelResult, CanFrame, Packet}, hardware::HardwareResult};
use egui::{Color32, plot::{Plot, Corner, Legend, Values}, Label, Sense};

use crate::{window::{InterfacePage, StatusBar, PageAction}, dyn_hw::DynHardware};

use super::status_bar::MainStatusBar;

// Common CAN Baud rates for OBD connection
const CAN_BAUD_RATES: &'static [u32] = &[
    5_000,
    10_000,
    20_000,
    31_250,
    33_333,
    40_000,
    50_000,
    80_000,
    83_333,
    100_000,
    125_000,
    200_000,
    250_000,
    500_000,
    1_000_000,
];

pub struct CanTracerPage {
    hw: DynHardware,
    channel: Option<Box<dyn CanChannel>>,
    status_bar: MainStatusBar,
    selected_baud: u32,
    error_maybe: Option<String>,
    can_map: HashMap<u32, CanFrame>,
    act_map: [f32; 100],
    events_draw: usize,
    mask: u32,
    mask_str: String,
    filt_str: String,
    filt: u32,
    max_y: f32
    //handle: Option<JoinHandle<()>>,
}

impl CanTracerPage {
    pub fn new(dev: DynHardware, bar: MainStatusBar) -> Self {
        Self {
            hw: dev,
            channel: None,
            status_bar: bar,
            max_y: 10.0,
            selected_baud: 500_000,
            error_maybe: None,
            can_map: HashMap::new(),//Arc::new(RwLock::new(HashMap::new())),
            act_map: [0f32; 100],
            events_draw: 0,
            mask: 0x0000,
            mask_str: "0000".into(),
            filt: 0x0000,
            filt_str: "0000".into()
            //handle: None
        }
    }

    pub fn open_can_channel(&mut self) -> ChannelResult<Box<dyn CanChannel>> {
        let mut channel = self.hw.create_can_channel()?;
        channel.set_can_cfg(self.selected_baud, false)?;
        channel.open()?;
        Ok(channel)
    }
}

impl InterfacePage for CanTracerPage {
    fn make_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::epi::Frame<'_>) -> crate::window::PageAction {
        if let Some(can_channel) = self.channel.borrow_mut() {
            let mut m = self.mask_str.clone();
            let mut f = self.filt_str.clone();
            ui.horizontal(|row| {
                row.label("Enter CAN ID Mask");
                row.text_edit_singleline(&mut m);
            });
            ui.horizontal(|row| {
                row.label("Enter CAN ID Filter");
                row.text_edit_singleline(&mut f);
            });
            self.filt_str = f;
            self.mask_str = m;
            if ui.button("Reset CAN filter").clicked() {
                self.filt_str = "0000".into();
                self.mask_str = "0000".into();
            }

            if !self.mask_str.is_empty() {
                if let Ok(parse) = u32::from_str_radix(&self.mask_str, 16) {
                    self.mask = parse;
                }
            }
            if !self.filt_str.is_empty() {
                if let Ok(parse) = u32::from_str_radix(&self.filt_str, 16) {
                    self.filt = parse;
                }
            }
            if ui.button("Disconnect").clicked() {
                self.channel.take();
                self.can_map.clear();
            } else {
                let start = Instant::now();
                let f = match can_channel.read_packets(1000, 15) {
                    Ok(mut v) => {
                        v.retain(|f| (f.get_address() & self.mask) == self.filt);
                        v
                    },
                    Err(e) => {
                        println!("Error reading : {}", e);
                        Vec::new()
                    }
                };
                let num = f.len() as f32;
                for frame in f {
                    self.can_map.insert(frame.get_address(), frame);
                }
                let dur = start.elapsed().as_millis();
                if num > self.max_y {
                    self.max_y = num;
                }
                if self.events_draw < 100 {
                    self.events_draw += 1;
                }
                self.act_map.rotate_right(1);
                self.act_map[0] = num;
                for f in self.can_map.values() {
                    if (f.get_address() & self.mask) == self.filt {
                        if ui.add(Label::new(format!("ID 0x{:04X}, DATA: {:02X?}", f.get_address(), f.get_data())).sense(Sense::click())).clicked() {
                            self.mask_str = "FFFF".into();
                            self.filt_str = format!("{:04X}", f.get_address());
                        }
                    }
                }
                let line = egui::plot::Line::new(Values::from_ys_f32(&self.act_map[0..self.events_draw]));
                let mut plot = Plot::new("Can activity")
                    .legend(
                        Legend::default().position(Corner::RightBottom)
                    )
                    .show_x(false)
                    .show_y(true)
                    //.data_aspect(0.5)
                    .allow_drag(false)
                    .allow_zoom(false)
                    .include_y(0.0)
                    .include_y(self.max_y+10.0)
                    .include_x(0.0)
                    .include_x(100.0)
                    .width(ui.available_width())
                    .line(line.name("CAN Events"));
                ui.add(plot);
                ui.ctx().request_repaint();
            }
        } else {
            egui::ComboBox::from_label("Select baud rate (bps)")
            .width(500.0)
            .selected_text(&self.selected_baud)
            .show_ui(ui, |cb_ui| {
                for x in CAN_BAUD_RATES {
                    cb_ui.selectable_value(&mut self.selected_baud, *x, x);
                }
            });
            if ui.button("Connect").clicked() {
                self.error_maybe = None;
                match self.open_can_channel() {
                    Ok(c) => {
                        self.channel = Some(c);
                    },
                    Err(e) => self.error_maybe = Some(format!("Could not open CAN channel!: {}", e)),
                }
            }
            if let Some(e) = &self.error_maybe {
                ui.colored_label(Color32::from_rgb(255,0,0), e);
            }
        }
        crate::window::PageAction::None
    }

    fn get_title(&self) -> &'static str {
        "OpenVehicleDiag CAN Tracer"
    }

    fn get_status_bar(&self) -> Option<Box<dyn crate::window::StatusBar>> {
        Some(Box::new(self.status_bar.clone()))
    }
}

impl Drop for CanTracerPage {
    fn drop(&mut self) {
        self.channel = None; // Drop this, it auto closes the channel! 
    }
}