use iced::{Application, Settings};
use lazy_static::lazy_static;
use libc;
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::ffi::*;
use std::sync::{Arc, RwLock};
use J2534Common::*;
mod commapi;
mod passthru;
mod themes;
mod windows;

use windows::window::MainWindow;

fn main() -> iced::Result {
    let mut launcher_settings = Settings::default();
    launcher_settings.window.resizable = true;
    launcher_settings.window.size = (1280, 720);
    launcher_settings.window.min_size = Some((1280, 720));
    MainWindow::run(launcher_settings)
}
