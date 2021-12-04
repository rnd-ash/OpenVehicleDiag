use std::{collections::VecDeque, borrow::BorrowMut};

use ecu_diagnostics::hardware::Hardware;
use eframe::{egui, epi};

use crate::pages::status_bar::{self, StatusBar};

pub struct MainWindow {
    pages: VecDeque<Box<dyn InterfacePage>>,
    curr_title: String,
    status_bar: Option<StatusBar>
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            pages: VecDeque::new(),
            curr_title: "OpenVehicleDiag".into(),
            status_bar: None
        }
    }
    pub fn add_new_page(&mut self, p: Box<dyn InterfacePage>) {
        self.pages.push_front(p)
    }

    pub fn pop_page(&mut self) {
        if let Some(p) = self.pages.pop_front() {
        }
    }
}

impl epi::App for MainWindow {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let mut show_status_bar = true;
        if self.pages.len() > 0 {
            show_status_bar = self.pages[0].show_status_bar();
            egui::TopBottomPanel::top("PAGE").show(ctx, |main_win_ui| {
                let layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
                main_win_ui.allocate_ui_with_layout(main_win_ui.available_size(), layout, |ui| {
                    match self.pages[0].make_ui(ui) {
                        PageAction::None => {},
                        PageAction::Destroy => self.pop_page(),
                        PageAction::Add(p) => self.add_new_page(p),
                    } 
                });
            });
        }
        if show_status_bar {
            if let Some(bar) = self.status_bar.borrow_mut() {
                egui::TopBottomPanel::bottom("NAV").show(ctx, |nav| {
                    bar.make_ui(nav)
                });
            }
        }
    }

    fn name(&self) -> &str {
        if self.pages.len() > 0 {
            self.pages[0].get_title()
        } else {
            "OpenVehicleDiag (EGUI edition)"
        }
    }
}

pub enum PageAction {
    None,
    Destroy,
    Add(Box<dyn InterfacePage>)
}

pub trait InterfacePage {
    fn make_ui(&mut self, ui: &mut egui::Ui) -> PageAction;
    fn get_title(&self) -> &'static str;
    fn show_status_bar(&self) -> bool;
}