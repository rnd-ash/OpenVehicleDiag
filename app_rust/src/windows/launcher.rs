use crate::passthru::{PassthruDevice, PassthruDrv};
use iced::{pick_list, button, Text, Row, Element, Radio, Align, Column, PickList, Container, Length, Button, Command, Image};
use crate::commapi::comm_api::{ComServerError, ComServer};
use crate::commapi::passthru_api::PassthruApi;
use crate::windows::window::{ApplicationError, WindowMessage};
use crate::windows::window::ApplicationError::DriverError;
use crate::windows::launcher::LauncherMessage::LaunchRequested;
use crate::themes::{button_coloured, ButtonType, button_outlined, picklist, container, radio_btn, text, TextType};
use crate::themes::elements::DropDown;

#[derive(Debug, Clone)]
pub struct Launcher {
    device_list_passthru: Vec<PassthruDevice>,
    device_names_passthru: Vec<String>,
    selected_device_passthru: String,

    selection: pick_list::State<String>,

    device_names_dpdu: Vec<String>,
    selected_device_dpdu: String,
    api_selection: API,

    launch_state: button::State,

    status_text: String

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum API {
    D_PDU,
    Passthru
}


#[derive(Debug, Clone)]
pub enum LauncherMessage {
    SwitchAPI(API),
    DeviceSelected(String),
    LaunchRequested,
    Start(Box<dyn ComServer>)
}

impl ToString for ApplicationError {
    fn to_string(&self) -> String {
        match self {
            ApplicationError::DriverError(x) => x.to_string()
        }
    }
}

type Result<T> = std::result::Result<T, ApplicationError>;
impl Launcher {

    pub fn new() -> Self {
        let passthru_devices = PassthruDevice::find_all().unwrap_or(Vec::new());
        let passthru_device_names: Vec<String> = passthru_devices.iter().map(|d| d.name.clone()).collect();
        let selected_passthru_device: String = passthru_device_names.get(0).map(|s| s.clone()).unwrap_or(String::new());

        Self {
            device_list_passthru: passthru_devices,
            device_names_passthru: passthru_device_names,
            selected_device_passthru: selected_passthru_device,
            device_names_dpdu: vec![],
            selected_device_dpdu: "".to_string(),
            selection: pick_list::State::default(),
            api_selection: API::Passthru,
            launch_state: button::State::default(),
            status_text: "".into(),
        }
    }

    pub fn update(&mut self, msg: &LauncherMessage) -> Option<WindowMessage> {
        match msg {
            LauncherMessage::SwitchAPI(api) => { self.api_selection = *api },
            LauncherMessage::DeviceSelected(d) => {
                if self.api_selection == API::Passthru {
                    self.selected_device_passthru = d.clone()
                } else {
                    self.selected_device_dpdu = d.clone()
                }
            }
            LauncherMessage::LaunchRequested => {
                if self.api_selection == API::Passthru {
                    match self.get_device_passthru() {
                        Ok((details, driver)) => {
                            let mut server = PassthruApi::new(details, driver);
                            if let Err(e) = server.open_device() {
                                self.status_text = e.to_string()
                            } else {
                                // Ready to launch OVD!
                                return Some(WindowMessage::StartApp(server.clone_box()))
                            }
                        },
                        Err(x) => {
                            self.status_text = x.to_string()
                        }
                    }
                } else {
                    // TODO D-PDU Launching
                }
            }
            _ => {}
        }
        None
    }

    pub fn view(&mut self) -> Element<LauncherMessage> {
        let selection = Row::new()
            .push(Text::new("API:"))
            .push(radio_btn(
                API::D_PDU,
                "D-PDU",
                Some(self.api_selection.clone()),
                LauncherMessage::SwitchAPI,
                ButtonType::Primary
            ))
            .push(radio_btn(
                API::Passthru,
                "Passthru",
                Some(self.api_selection.clone()),
                LauncherMessage::SwitchAPI,
                ButtonType::Primary
            ))
            .padding(20)
            .spacing(10)
            .align_items(Align::Center);

        let contents = if self.api_selection == API::D_PDU {
            Column::new()
                .push(Image::new("img/logo.png").width(Length::Units(300)).height(Length::Units(300)))
                .push(selection)
                .push(Text::new("D-PDU API is unimplemented, check back in a future release!"))
                .spacing(10)
        } else {
            let mut c = Column::new()
                .push(Image::new("img/logo.png").width(Length::Units(300)).height(Length::Units(300)))
                .spacing(10)
                .padding(20)
                .push(selection);
            if self.selected_device_passthru.len() == 0 {
                // No passthru devices
                c = c.push(text("No Passthru devices found on this system", TextType::Normal))
            } else {
                c = c.push(Text::new("Select Passthru device"))
                    .push(picklist(
                        &mut self.selection,
                        &self.device_names_passthru,
                        Some(self.selected_device_passthru.clone()),
                        LauncherMessage::DeviceSelected))
                    //.push(Button::new(&mut self.launch_state, Text::new("Launch OVD!"))
                    //    .on_press(LaunchRequested).style(MaterialButtonOutline)
                    .push(button_coloured(&mut self.launch_state, "Launch OVD!", ButtonType::Primary).on_press(LaunchRequested))
                    .push(Text::new(&self.status_text));
            }
            c.align_items(Align::Center)
        };
        container(contents).center_x().width(Length::Fill).height(Length::Fill).into()
    }

    fn get_device_passthru(&self) -> Result<(PassthruDevice, PassthruDrv)> {
        match self.device_list_passthru.iter().find(|d| d.name == self.selected_device_passthru) {
            Some(d) => match PassthruDrv::load_lib(d.drv_path.clone()) {
                Ok(lib) =>Ok((d.clone(), lib)),
                Err(e) => Err(DriverError(ComServerError{
                    err_code: 99,
                    err_desc: format!("Cannot locate driver at {}", d.drv_path)
                })),
            },
            // This should NEVER happen.
            None => Err(DriverError(ComServerError{
                err_code: 99,
                err_desc: format!("WTF. Located device is not valid??")
            }))
        }
    }
}