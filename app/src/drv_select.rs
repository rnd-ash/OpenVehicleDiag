extern crate gtk;
use gtk::*;
use std::fs;
use std::io::*;
use std::rc::*;
use std::cell::RefCell;
use crate::passthru::*;
use crate::ovd::*;

pub struct DrvSelect {
    pub window: Window,
    pub header: Header,
    pub content: Content,
}

pub struct Header {
    pub container: HeaderBar,
}

pub struct Content {
    pub container: Box,
    pub text: Label,
    pub dropdown: ComboBoxText,
    pub ok_btn: Button,
    pub devices: Option<Vec<PassthruDevice>>,
    pub launched: bool,
    //pub dev_index: Rc<RefCell<usize>>,
}

impl Content {
    fn new() -> Content {
        let container = Box::new(Orientation::Vertical, 10);
        let devices = PassthruDevice::find_all();
        let dropdown = ComboBoxText::new();
        let text = Label::new(Some("Select Device (J2534 compatible)"));
        let desc = Label::new(None);

        let ok_btn = Button::new();
        ok_btn.set_label("Launch OVD");
        ok_btn.set_sensitive(false);
        ok_btn.set_halign(Align::End); 
        let mut dev_list : Option<Vec<PassthruDevice>> = None;
        match devices {
            Err(x) => {
                text.set_text(format!("Error. No devices found! ({:?})", x).as_str());
                dropdown.set_sensitive(false);
            },
            Ok(ls) => {

                for dev in &ls {
                    dropdown.append_text(format!("{} ({})",dev.name, dev.vendor).as_str())
                }
                dev_list = Some(ls);
                ok_btn.connect_clicked(move |_| {
                    let o = OvdApp::new();
                    o.window.show_all();
                    //main_quit();
                });
            }
        }
        
        container.pack_start(&text, false, false, 10);
        container.pack_start(&desc, false, false, 10);
        container.pack_start(&Separator::new(Orientation::Horizontal), false, false, 0);
        container.pack_start(&dropdown, false, false, 10);
        container.pack_end(&ok_btn, false, false, 10);

        
        
        let btn_clone = ok_btn.clone();
        dropdown.connect_changed(move |x| {
            btn_clone.set_sensitive(true);
        });


        Content { container, text, dropdown, ok_btn, devices: dev_list, launched: false}
    }
}

impl Header {
    fn new() -> Header {
        let container = HeaderBar::new();
        container.set_title(Some("Select Passthru device"));
        container.set_show_close_button(true);
        Header { container }
    }
}

impl DrvSelect {
    pub fn new() -> DrvSelect {
        let window = Window::new(WindowType::Toplevel);
        let header = Header::new();
        let content = Content::new();
        window.set_titlebar(Some(&header.container));
        window.add(&content.container);
        window.set_title("Select Passthru device");
        window.set_wmclass("ovd", "Open Vehicle Diagnostics");
        window.set_default_size(480, 240);
        window.set_resizable(false);
        if let Err(_) = window.set_icon_from_file("icon.png") {
            eprintln!("Error setting icon!");
        }
        window.connect_delete_event(move |_,_| {
            main_quit();
            Inhibit(false)
        });
        return DrvSelect { window, header, content }
    }
}