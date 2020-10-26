extern crate gtk;
use gtk::*;

pub struct OvdApp {
    pub window: Window,
    pub header: OvdHeader
}

pub struct OvdHeader {
    pub container: HeaderBar,
    pub voltage: Label,
}

impl OvdHeader {
    fn new() -> OvdHeader {
        let container = HeaderBar::new();
        container.set_title(Some("Open Vehicle Diagnostics"));
        container.set_show_close_button(true);
        let voltage = Label::new(Some("Battery: 12.0V"));
        container.pack_end(&voltage);
        OvdHeader { container, voltage }
    }
}

impl OvdApp {
    pub fn new() -> OvdApp {
        let window = Window::new(WindowType::Toplevel);
        let header = OvdHeader::new();
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
        return OvdApp { window, header }
    }
}