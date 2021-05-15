use std::{fs::File, io::Write, panic, time};
use backtrace;

use dialog::DialogBox;
use iced::{Application, Settings};
mod cli_tests;
mod commapi;
mod passthru;
mod themes;
mod widgets;
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
            themes::set_debug(true)
        }
    }

    panic::set_hook(Box::new(|info|{
        let backtrace = backtrace::Backtrace::new();

        let mut report = String::from("!!! Report crash to https://github.com/rnd-ash/OpenVehicleDiag/issues/ !!!\n");
        report.push_str("\n----\n");
        report.push_str(format!("Reason: {}\nBacktrace:\n{:?}", info, backtrace).as_str());
        report.push_str("\n----\n");
        let time = chrono::Utc::now();
        let path = std::env::current_dir().unwrap().join(format!("ovd_crash-{}.txt", time.format("%F-%H_%M_%S")));
        let write_res = File::create(&path).unwrap().write_all(report.as_bytes());

        let mut summary = format!("Reason: {}\n", info);
        summary.push_str(format!("Crash report written to {}\n", &path.as_os_str().to_str().unwrap()).as_str());
        summary.push_str("See crash report for more info on how to report");

        let _res = dialog::Message::new(summary)
            .title("Oh no! OpenVehicleDiag crashed")
            .show();
    }));

    MainWindow::run(launcher_settings)
}
