extern crate gtk;
use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageType, MessageDialog, Window, Settings};
mod drv_select;
mod passthru;
mod ovd;
mod log;

fn main() {
    gtk::init().expect("Error loading GTK!");

    let mut app = drv_select::DrvSelect::new();


    get_theme();

    app.window.show_all();
    gtk::main();
}

#[cfg(windows)]
fn get_theme() {
    gtk::Settings::get_default().unwrap().set_property_gtk_theme_name(Some("win32"));
}

#[cfg(unix)] // Nothing on linux - use native GTK
fn get_theme(){}