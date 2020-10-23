extern crate gtk;
use gtk::*;
mod app;
mod drv_select;
mod passthru;

fn main() {
    gtk::init().expect("Error loading GTK!");

    let mut app = drv_select::DrvSelect::new();
    app.window.show_all();
    println!("{:?}", app.get_selected_device());
    gtk::main();
}