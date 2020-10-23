extern crate gtk;
use gtk::*;
use std::fs;
use std::io::*;
use std::rc::*;
use std::cell::RefCell;
use crate::passthru::*;

#[cfg(unix)]
use serde_json;

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
    pub devices: Option<Vec<PassthruDevice>>
}

impl Content {
    fn new() -> Content {
        let container = Box::new(Orientation::Vertical, 10);
        let devices = get_pt_list();
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
                text.set_text(format!("Error. No devices found! ({})", x).as_str());
                dropdown.set_sensitive(false);
            },
            Ok(ls) => {

                for dev in &ls {
                    dropdown.append_text(format!("{} ({})",dev.name, dev.vendor).as_str())
                }
                dev_list = Some(ls);
                ok_btn.connect_clicked(move |_| {
                    main_quit();
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
        Content { container, text, dropdown, ok_btn, devices: dev_list}
    }

    fn get_device(&mut self) -> Option<PassthruDevice> {
        None
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

    pub fn get_selected_device(&mut self) -> Option<PassthruDevice> {
        self.content.get_device()
    }
}

#[derive(Debug)]
pub struct PassthruDevice {
    /// Does the device supports CAN?
    can: bool,
    /// Does the device support ISO15765 over CAN?
    iso15765: bool,
    /// Does the device support ISO9141 over K-Line?
    iso9141: bool,
    /// Does the device support ISO14230 over K-Line?
    iso14230: bool,
    /// Does the device support J1850-VPW?
    j1850vpw: bool,
    /// Does the device support J1850-PWM?
    j1850pwm: bool,
    /// Path to library to load for the device
    function_lib: String,
    /// Name of the device
    name: String,
    /// Device vendor name
    vendor: String,
    /// Device driver
    drv: PassthruDrv
}

#[cfg(unix)]
fn get_pt_list<'a>() -> Result<Vec<PassthruDevice>> {
    // Ensure passthru JSON directory exists
    if !std::path::Path::new("/usr/share/passthru/").is_dir() {
        return Err(Error::new(ErrorKind::NotFound, "Path not found"));
    }

    let mut found_devices : Vec<PassthruDevice> = Vec::new();
    fs::read_dir("/usr/share/passthru/")?.for_each(|x| {
        if let Ok(f) = x {
            if f.file_name().to_str().unwrap().contains(".json") {
                println!("PTLOCATE -> Found {}", f.file_name().to_str().unwrap());
                if let Ok(mut file) = std::fs::File::open(f.path()) {
                    let mut buf = String::new();
                    file.read_to_string(&mut buf).unwrap();
                    if let Some(dev) =  read_device(&buf) {
                        found_devices.push(dev);
                    }
                }
            }
        }
    });
    if !found_devices.is_empty() {
        return Ok(found_devices);
    }
    Err(Error::new(ErrorKind::NotFound, "No JSON in /usr/share/passthru/ found"))
}


#[cfg(unix)]
fn get_json_bool(k: &str, value: &serde_json::Value) -> bool {
    if let serde_json::Value::Bool(x) = value[k] {
        return x
    }
    false
}

#[cfg(unix)]
fn read_device(s: &String) -> Option<PassthruDevice> {
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(s.as_str()) {
        let name = json["NAME"].as_str();
        let f_lib = json["FUNCTION_LIB"].as_str();
        let vend = json["VENDOR"].as_str();
        // This is required!
        if name == None || f_lib == None || vend == None {
            eprintln!("PTLOCATE -> JSON does not have either NAME, FUNCTION_LIB, or VENDOR field");
            return None;
        }
        println!("Loading library from {}", f_lib.unwrap().to_string());
        let lib = PassthruDrv::load_lib(f_lib.unwrap().to_string());
        if let Ok(l) = lib {
            if let Ok(v) = l.get_version() {
                println!("PTLOCATE -> Library Load OK!");
                println!("FW Version: {}", v.fw_version);
                println!("API Version: {}", v.api_version);
                println!("DLL Version: {}", v.dll_version);
            }
            return Some(PassthruDevice {
                name: name.unwrap().to_string(),
                vendor: vend.unwrap().to_string(),
                can: get_json_bool("CAN", &json),
                iso14230: get_json_bool("ISO14230", &json),
                iso15765: get_json_bool("ISO15765", &json),
                iso9141: get_json_bool("ISO9141", &json),
                j1850vpw: get_json_bool("J1850VPW", &json),
                j1850pwm: get_json_bool("J1850PWM", &json),
                function_lib: f_lib.unwrap().to_string(),
                drv: l
            })
        } else {
            eprintln!("ERROR Loading library {:?}", lib.err());
        }
    }
    None
}

#[cfg(windows)]
fn get_pt_list() -> Option<Vec<PassthruDevice>> {
    None
    // TODO
}