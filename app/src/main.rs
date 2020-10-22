extern crate gtk;
use gtk::*;
use std::process;
mod app;

fn main() {
    gtk::init().expect("Error loading GTK!");

    let app = app::App::new();
    app.window.show_all();
    gtk::main();

}