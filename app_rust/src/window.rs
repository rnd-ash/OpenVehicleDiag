use std::{collections::VecDeque, borrow::BorrowMut};

use ecu_diagnostics::hardware::Hardware;
use eframe::{egui, epi};

use crate::{pages::status_bar::{self}, dyn_hw::DynHardware};

pub struct MainWindow {
    pages: VecDeque<Box<dyn InterfacePage>>,
    curr_title: String,
    bar: Option<Box<dyn StatusBar>>
}

impl MainWindow {
    pub fn new() -> Self {
        Self {
            pages: VecDeque::new(),
            curr_title: "OpenVehicleDiag".into(),
            bar: None
        }
    }
    pub fn add_new_page(&mut self, p: Box<dyn InterfacePage>) {
        if let Some(bar) = p.get_status_bar() {
            self.bar = Some(bar)
        }
        self.pages.push_front(p)
    }

    pub fn pop_page(&mut self) {
        self.pages.pop_front();
        if let Some(bar) = self.pages.get_mut(0).map(|x| x.get_status_bar()) {
            self.bar = bar
        }
    }
}

impl epi::App for MainWindow {
    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        let stack_size = self.pages.len();
        if stack_size > 0 {
            let mut pop_page = false;
            if let Some(status_bar) = self.bar.borrow_mut() {
                egui::TopBottomPanel::bottom("NAV").default_height(800.0).show(ctx, |nav| {
                    status_bar.draw(nav);
                    if stack_size > 1 {
                        if nav.button("Back").clicked() {
                            pop_page = true;
                        }
                    }
                });
            }
            if pop_page {
                self.pop_page();
            }
            egui::CentralPanel::default().show(ctx, |main_win_ui| {
                match self.pages[0].make_ui(main_win_ui, frame) {
                    PageAction::None => {},
                    PageAction::Destroy => self.pop_page(),
                    PageAction::Add(p) => self.add_new_page(p),
                    PageAction::Overwrite(p) => {
                        self.pages[0] = p;
                        self.bar = self.pages[0].get_status_bar();
                    },
                    PageAction::RePaint => { ctx.request_repaint() }
                }
            });
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
    Add(Box<dyn InterfacePage>),
    Overwrite(Box<dyn InterfacePage>),
    RePaint,
}

pub trait InterfacePage {
    fn make_ui(&mut self, ui: &mut egui::Ui, frame: &mut epi::Frame<'_>) -> PageAction;
    fn get_title(&self) -> &'static str;
    fn get_status_bar(&self) -> Option<Box<dyn StatusBar>>;
}

pub trait StatusBar {
    fn draw(&mut self, ui: &mut egui::Ui);
}