use iced::{Application, Settings};
mod commapi;
mod passthru;
mod themes;
mod windows;

use windows::window::MainWindow;
use iced::window::Icon;
use image::GenericImageView;

fn main() -> iced::Result {
    let mut launcher_settings = Settings::default();
    launcher_settings.window.resizable = true;
    launcher_settings.window.size = (1280, 720);
    launcher_settings.window.min_size = Some((1280, 720));
    if let Ok(img) = image::open("./img/launcher.png") {
        launcher_settings.window.icon = Icon::from_rgba(img.clone().into_bytes(), img.width(), img.height()).ok()
    }

    let args = std::env::args();
    for a in args {
        if a == "-debug_ui" {
            themes::setDebug(true)
        }
    }


    MainWindow::run(launcher_settings)
}
