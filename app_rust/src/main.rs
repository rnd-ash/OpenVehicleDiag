use iced::{Application, Settings, Image};
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
use std::io::Read;
use iced::window::Icon;
use image::GenericImageView;

fn main() -> iced::Result {
    let mut launcher_settings = Settings::default();
    launcher_settings.window.resizable = true;
    launcher_settings.window.size = (1280, 720);
    launcher_settings.window.min_size = Some((1280, 720));
    if let Ok(mut img) = image::open("./img/launcher.png") {
        launcher_settings.window.icon = Icon::from_rgba(img.clone().into_bytes(), img.width(), img.height()).ok()
    }

    MainWindow::run(launcher_settings)
}
