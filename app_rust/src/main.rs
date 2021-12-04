use std::{fs::File, io::Write, panic, time};
use backtrace;
use dialog::DialogBox;
use eframe::epi::IconData;
use image::{GenericImageView, ImageFormat};

mod resources;
mod window;
mod pages;
mod dyn_hw;

use pages::launcher::Launcher;
use resources::TRAY_ICON;
use window::*;

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
    eframe::run_native(Box::new(app), native_options)

    
}
