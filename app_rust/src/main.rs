use std::{fs::File, io::Write, panic, time};
use backtrace;
use dialog::DialogBox;
use eframe::epi::IconData;
use egui::Vec2;
use image::{GenericImageView, ImageFormat};

mod resources;
mod window;
mod pages;
mod dyn_hw;

use pages::launcher::Launcher;
use resources::TRAY_ICON;
use window::*;

// IMPORTANT. On windows, only the i686-pc-windows-msvc target is supported (Due to limitations with J2534 and D-PDU!
#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
compile_error!("Windows can ONLY be built using the i686-pc-windows-msvc target!");


fn main() {
    let mut app = MainWindow::new();
    let mut native_options = eframe::NativeOptions::default();
    if let Ok(img) = image::load_from_memory_with_format(TRAY_ICON, ImageFormat::Png) {
        native_options.icon_data = Some(IconData {
            rgba: img.clone().into_bytes(),
            width: img.width(),
            height: img.height(),
        })
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
    app.add_new_page(Box::new(Launcher::new()));
    native_options.initial_window_size = Some(Vec2::new(800.0, 600.0));
    eframe::run_native(Box::new(app), native_options)

    
}
