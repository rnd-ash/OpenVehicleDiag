use iced::{Application, Settings};
mod cli_tests;
mod commapi;
mod graphs;
mod passthru;
mod themes;
mod windows;

use iced::window::Icon;
use image::{GenericImageView, ImageFormat};
use themes::images::TRAY_ICON;
use windows::window::MainWindow;

pub const WIN_WIDTH: u32 = 1600;
pub const WIN_HEIGHT: u32 = 900;

fn main() -> iced::Result {
    let mut launcher_settings = Settings::default();
    launcher_settings.window.resizable = false;
    launcher_settings.window.size = (WIN_WIDTH, WIN_HEIGHT);
    launcher_settings.window.min_size = Some((WIN_WIDTH, WIN_HEIGHT));

    if let Ok(img) = image::load_from_memory_with_format(TRAY_ICON, ImageFormat::Png) {
        launcher_settings.window.icon =
            Icon::from_rgba(img.clone().into_bytes(), img.width(), img.height()).ok()
    }

    let args = std::env::args();
    for a in args {
        if a == "-debug_ui" {
            themes::setDebug(true)
        }
    }
    MainWindow::run(launcher_settings)
}
