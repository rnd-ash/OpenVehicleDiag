use std::fmt::{Debug};
use iced::{button, executor, Align, Application, Column, Command, Element, Length, Row, Text, Subscription, time, Rule, Space};
use crate::commapi::comm_api::{ComServer, ComServerError};
use crate::windows::launcher::{Launcher, LauncherMessage};
use crate::windows::home::{Home, HomeMessage};
use std::time::Instant;
use crate::windows::cantracer::{CanTracer, TracerMessage};
use crate::windows::uds_scanner::{UDSHomeMessage, UDSHome};
use crate::windows::obd::{OBDMessage, OBDHome};
use crate::themes::{toggle_theme, button_coloured, ButtonType, container, text, TextType};

#[derive(Debug, Clone)]
pub (crate) enum ApplicationError {
    DriverError(ComServerError)
}

#[derive(Debug, Clone)]
pub enum WindowState {
    Launcher(Launcher),
    Home(Home),
    CanTracer(CanTracer),
    UDSHome(UDSHome),
    OBDTools(OBDHome),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowStateName {
    Launcher,
    Home,
    CanTracer,
    UDSHome,
    OBDTools,
}

impl<'a> WindowState {
    fn view(&mut self) -> Element<WindowMessage> {
        match self {
            Self::Launcher(launcher) => launcher.view().map(WindowMessage::Launcher),
            Self::Home (home) => home.view(),
            Self::CanTracer (tracer) => tracer.view().map(WindowMessage::CanTracer),
            Self::UDSHome (h) => h.view().map(WindowMessage::UDSScanner),
            Self::OBDTools (h) => h.view().map(WindowMessage::OBDTools),
        }
    }

    fn update(&mut self, msg: &WindowMessage) -> Option<WindowMessage> {
        match self {
            Self::Launcher(launcher) => {
                if let WindowMessage::Launcher(x) = msg {
                    return launcher.update(x);
                }
            },
            Self::Home (home) => {
                if let WindowMessage::Home(x) = msg {
                    return home.update(x);
                }
            },
            Self::CanTracer (tracer) => {
                if let WindowMessage::CanTracer(x) = msg {
                    return tracer.update(x);
                }
            },
            Self::UDSHome (uds) => {
                if let WindowMessage::UDSScanner(x) = msg {
                    return uds.update(x).map(WindowMessage::UDSScanner)
                }
            },
            Self::OBDTools(o) => {
                if let WindowMessage::OBDTools(x) = msg {
                    return o.update(x).map(WindowMessage::OBDTools)
                }
            }
        }
        None
    }
}

impl WindowState {
    fn get_name(&self) -> WindowStateName {
        match &self {
            WindowState::Launcher { .. } => WindowStateName::Launcher,
            WindowState::Home { .. } => WindowStateName::Home,
            WindowState::CanTracer { .. } => WindowStateName::CanTracer,
            WindowState::UDSHome { .. } => WindowStateName::UDSHome,
            WindowState::OBDTools { .. } => WindowStateName::OBDTools,
        }
    }
}



#[derive(Debug, Clone)]
pub enum WindowMessage {
    Launcher(LauncherMessage),
    Home(HomeMessage),
    CanTracer(TracerMessage),
    UDSScanner(UDSHomeMessage),
    OBDTools(OBDMessage),
    StartApp(Box<dyn ComServer>),
    StatusUpdate(Instant),
    GoHome, // Goto home page
    GoCanTracer, // Goto Can Tracer page
    GoUDS, // Goto UDS Scanner page
    GoOBD, // Goto OBD Toolbox page
    ToggleTheme, // Toggle the theme
}


pub struct MainWindow {
    state: WindowState,
    server: Option<Box<dyn ComServer>>,
    voltage: f32,
    back_btn_state: button::State,
    theme_toggle: button::State,
}

impl Application for MainWindow {
    type Executor = executor::Default;
    type Message = WindowMessage;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {
            state: WindowState::Launcher(Launcher::new()),
            server: None,
            voltage: 0.0,
            back_btn_state: button::State::default(),
            theme_toggle: button::State::default()
        }, Command::none())
    }

    fn title(&self) -> String {
        match &self.state {
            WindowState::Launcher { .. } => "OpenVehicleDiag launcher".into(),
            WindowState::Home { .. } => format!("OpenVehicleDiag ({} mode)", self.server.as_ref().map(|s| s.get_api()).unwrap_or("Unknown")),
            WindowState::CanTracer { .. } => "OpenVehicleDiag CanTracer".into(),
            WindowState::UDSHome { .. } => "OpenVehicleDiag UDS Scanner".into(),
            WindowState::OBDTools { .. } => "OpenVehicleDiag OBD Toolbox".into()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<WindowMessage> {
        match message {
            WindowMessage::StatusUpdate(_) => {
                // On request for battery voltage reading, try to read from the adapter, but it might timeout
                // if the driver is under heady IO load, so then use the current voltage reading
                self.voltage = self.server.as_ref().unwrap().read_battery_voltage().unwrap_or(self.voltage)
            },
            WindowMessage::GoHome => {
                self.state = WindowState::Home(Home::new(self.server.clone().unwrap()))
            },
            WindowMessage::GoCanTracer => {
                self.state = WindowState::CanTracer(CanTracer::new(self.server.clone().unwrap()))
            },
            WindowMessage::GoUDS => {
                self.state = WindowState::UDSHome(UDSHome::new(self.server.clone().unwrap()))
            },
            WindowMessage::GoOBD => {
                self.state = WindowState::OBDTools(OBDHome::new(self.server.clone().unwrap()))
            }
            WindowMessage::ToggleTheme => {
                toggle_theme()
            }
            _ => return self.update_children(&message)
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if let WindowState::Launcher { .. } = self.state {
            Subscription::none()
        } else {
            // Ask for battery every 2 seconds
            let mut batch: Vec<Subscription<WindowMessage>> = vec![];

            batch.push(time::every(std::time::Duration::from_secs(2)).map(WindowMessage::StatusUpdate));
            // See if either other pages request update
            if let WindowState::CanTracer(tracer) = &self.state {
                batch.push(tracer.subscription().map(WindowMessage::CanTracer))
            } else if let WindowState::UDSHome(uds) = &self.state {
                batch.push(uds.subscription().map(WindowMessage::UDSScanner))
            }
            Subscription::batch(batch)
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // If not in launcher mode we should draw the bottom status bar as well
        return if let WindowState::Launcher { .. } = self.state {
            self.state.view()
        } else {
            // Draw the status bar!
            let t = match self.server.as_ref().unwrap().is_connected() {
                true => text("Connected", TextType::Success),
                false => text("Disconnected", TextType::Danger),
            };

            let v = if self.voltage < 12.0 && self.voltage > 11.5 {
                text(format!("{}V", self.voltage).as_str(), TextType::Warning)
            } else if self.voltage < 11.5 {
                text(format!("{}V", self.voltage).as_str(), TextType::Danger)
            } else {
                text(format!("{}V", self.voltage).as_str(), TextType::Success)
            };
            let page_name = &self.state.get_name();
            let view_contents = self.state.view();
            let mut s_bar = Row::new().padding(5).spacing(5).height(Length::Shrink)
                    .push(Row::new().spacing(5)
                    .push(Text::new("Status: "))
                    .push(t)
                    .align_items(Align::Center)
                )
                .push(Space::with_width(Length::Units(50)))
                .push(Row::new()
                    .spacing(5)
                    .push(Text::new("Battery voltage: "))
                    .push(v)
                ).push(Space::with_width(Length::Fill)
                );

            let mut btn_row = Row::new().spacing(5)
                .push(button_coloured(&mut self.theme_toggle, "Toggle theme", ButtonType::Secondary)
                    .on_press(WindowMessage::ToggleTheme));

            if page_name != &WindowStateName::Home {
                btn_row = btn_row.push(button_coloured(&mut self.back_btn_state,"Go home", ButtonType::Warning)
                    .on_press(WindowMessage::GoHome)
                )
            }
            s_bar = s_bar.push(btn_row);
            let c = Column::new()
                .push(view_contents)
                .push(Space::with_height(Length::Fill))
                .push(Rule::horizontal(2))
                .push(s_bar)
                .height(Length::Fill)
                .width(Length::Fill);
            container(c).into()
        }
    }
}

impl MainWindow {
    fn update_children(&mut self, message: &WindowMessage) -> Command<WindowMessage> {
        // Special case handling
        if let Some(state) = self.state.update(message) {
            match state {
                WindowMessage::StartApp(srv) => {
                    self.server = Some(srv.clone_box());
                    self.voltage = self.server.as_ref().unwrap().read_battery_voltage().unwrap_or(0.0);
                    self.state = WindowState::Home(Home::new(srv));
                    Command::none()
                },
                _ => Command::perform(async move { state.clone() }, |x| x)
            }
        } else {
            Command::none()
        }
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        if let Some(mut s) = self.server.take() {
            s.close_iso15765_interface();
            s.close_can_interface();
            s.close_device();
        }
    }
}
