use std::fmt::format;

use crate::{comserver, passthru::*};
use comserver::ComServer;
use iced::{
    button, executor, pick_list, Align, Application, Button, Column, Command, Container, Element,
    Length, PickList, Row, Settings, Text,
};
use J2534Common::Loggable;

#[derive(Debug, Default)]
pub struct Launcher {
    devices: Vec<PassthruDevice>,
    device_list: pick_list::State<String>,
    device_names: Vec<String>,
    btn_state: button::State,
    selected_device: String,
    summary_text: String,
    server: Option<ComServer>
}

#[derive(Debug, Clone)]
pub enum Message {
    DeviceSelected(String),
    Launch,
}

impl Application for Launcher {
    type Executor = executor::Default;

    type Message = Message;

    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let devices = match PassthruDevice::find_all() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error locating Passthru devices {:?}", e);
                Vec::new()
            }
        };
        let device_names: Vec<String> = devices.iter().map(|dev| dev.name.clone()).collect();

        let mut selected_device: String = String::default();
        if device_names.len() > 0 {
            selected_device = device_names[0].clone();
        }

        (
            Self {
                devices,
                device_names,
                selected_device,
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("OpenVehicleDiag launcher")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DeviceSelected(name) => self.selected_device = name,
            Message::Launch => {
                println!(
                    "Launch requested. Selected device: {}",
                    self.selected_device
                );
                // Try to connect with the device
                match self.load_device() {
                    Ok((details, driver, idx)) => {
                        println!("Ready to launch!");
                        let mut s = comserver::ComServer::new(details, driver, idx);
                        s.create_can_channel(500_000, false);
                        println!("{}", s.get_batt_voltage().unwrap());
                        
                        // Create main window
                    }
                    Err(e) => self.summary_text = e,
                }
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let contents = if self.devices.len() > 0 {
            Column::new()
                .spacing(20)
                .padding(40)
                .max_width(800)
                .align_items(Align::Center)
                .push(Text::new("Select passthru device".to_string()))
                .push(PickList::new(
                    &mut self.device_list,
                    &self.device_names,
                    Some(self.selected_device.clone()),
                    Message::DeviceSelected,
                ))
                .push(
                    Button::new(&mut self.btn_state, Text::new("Launch OVD"))
                        .on_press(Message::Launch),
                )
                .push(Text::new(&self.summary_text))
        } else {
            Column::new()
                .spacing(20)
                .padding(40)
                .max_width(800)
                .align_items(Align::Center)
                .push(Text::new("No passthru located".to_string()))
                .push(Text::new("please consult the manual".to_string()))
        };
        Container::new(contents)
            .width(Length::Fill)
            .center_x()
            .into()
    }
}

type Result<T> = std::result::Result<T, String>;
impl Launcher {
    fn load_device(&self) -> Result<(PassthruDevice, PassthruDrv, u32)> {
        match self.devices.iter().find(|d| d.name == self.selected_device) {
            Some(d) => match PassthruDrv::load_lib(d.drv_path.clone()) {
                Ok(mut dev) => match dev.open() {
                    Ok(idx) => Ok((d.clone(), dev, idx)),
                    Err(e) => Err(format!(
                        "Driver error when opening device: {}",
                        e.to_string()
                    )),
                },
                Err(e) => Err(format!("Failed to open device library {:?}", e)),
            },
            None => Err(format!("Unable to locate device!?")),
        }
    }
}
