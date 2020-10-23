extern crate gtk;
use gtk::*;

pub struct App {
    pub window: Window,
    pub header: Header
}

pub struct Header {
    pub container: HeaderBar,
    pub voltage: Label,
}

impl Header {
    fn new() -> Header {
        let container = HeaderBar::new();
        container.set_title(Some("Open Vehicle Diagnostics"));
        container.set_show_close_button(true);
        let voltage = Label::new(Some("Battery: 12.0V"));
        container.pack_end(&voltage);
        Header { container, voltage }
    }
}

impl App {
    pub fn new() -> App {
        let window = Window::new(WindowType::Toplevel);
        let header = Header::new();
        window.set_titlebar(Some(&header.container));
        window.set_title("Open Vehicle Diagnostics");
        window.set_wmclass("ovd", "Open Vehicle Diagnostics");
        window.set_default_size(1280, 720);
        if let Err(_) = window.set_icon_from_file("icon.png") {
            eprintln!("Error setting icon!");
        }
        window.connect_delete_event(move |_,_| {
            main_quit();
            Inhibit(false)
        });
        return App { window, header }
    }
}