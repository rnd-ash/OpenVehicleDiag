use std::fmt::{format, Debug};
use crate::{commapi, passthru::*};
use iced::{button, executor, pick_list, Align, Application, Button, Column, Command, Container, Element, Length, PickList, Row, Settings, Text, Sandbox, Radio, Subscription, Color, time, Rule, Space};
use J2534Common::Loggable;
use crate::commapi::passthru_api::PassthruApi;
use crate::commapi::comm_api::{ComServer, FilterType, ComServerError};
use crate::windows::window::ApplicationError::DriverError;
use serde::export::Formatter;
use crate::windows::window::LauncherMessage::LaunchRequested;
use crate::windows::launcher::{Launcher, LauncherMessage};
use crate::windows::home::{Home, HomeMessage};
use std::time::Instant;
use crate::windows::cantracer::{CanTracer, TracerMessage};
use std::ops::Sub;
use crate::windows::uds_scanner::{UDSHomeMessage, UDSHome};

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
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowStateName {
    Launcher,
    Home,
    CanTracer,
    UDSHome,
}

impl<'a> WindowState {
    fn view(&mut self) -> Element<WindowMessage> {
        match self {
            Self::Launcher(launcher) => launcher.view().map(|x| WindowMessage::Launcher(x)).into(),
            Self::Home (home) => home.view().into(),
            Self::CanTracer (tracer) => tracer.view().map(|x| WindowMessage::CanTracer(x)).into(),
            Self::UDSHome (h) => h.view().map(|x| WindowMessage::UDSScanner(x)).into(),
            _ => unimplemented!()
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
                    return uds.update(x).map(|t| WindowMessage::UDSScanner(t))
                }
            }
            _ => unimplemented!()
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
        }
    }
}



#[derive(Debug, Clone)]
pub enum WindowMessage {
    Launcher(LauncherMessage),
    Home(HomeMessage),
    CanTracer(TracerMessage),
    UDSScanner(UDSHomeMessage),
    StartApp(Box<dyn ComServer>),
    LaunchFileBrowser,
    StatusUpdate(Instant),
    GoHome, // Goto home page
    GoCanTracer, // Goto Can Tracer page
    GoUDS, // Goto UDS Scanner page
}


pub struct MainWindow {
    state: WindowState,
    server: Option<Box<dyn ComServer>>,
    voltage: f32,
    back_btn_state: button::State
}

impl Application for MainWindow {
    type Executor = executor::Default;
    type Message = WindowMessage;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {
            state: WindowState::Launcher(Launcher::new()),
            server: None,
            voltage: 0.0,
            back_btn_state: button::State::default()
        }, Command::none())
    }

    fn title(&self) -> String {
        match &self.state {
            WindowState::Launcher { .. } => "OpenVehicleDiag launcher".into(),
            WindowState::Home { .. } => format!("OpenVehicleDiag ({} mode)", self.server.as_ref().map(|s| s.get_api()).unwrap_or("Unknown")),
            WindowState::CanTracer { .. } => "OpenVehicleDiag CanTracer".into(),
            WindowState::UDSHome { .. } => "OpenVehicleDiag UDS Scanner".into()
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
            }
            _ => return self.update_children(&message)
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if let WindowState::Launcher { .. } = self.state {
            return Subscription::none();
        } else {
            // Ask for battery every 2 seconds
            let mut batch: Vec<Subscription<WindowMessage>> = vec![];

            batch.push(time::every(std::time::Duration::from_secs(2)).map(WindowMessage::StatusUpdate));
            // See if either other pages request update
            if let WindowState::CanTracer(tracer) = &self.state {
                batch.push(tracer.subscription().map(|x| WindowMessage::CanTracer(x)))
            } else if let WindowState::UDSHome(uds) = &self.state {
                batch.push(uds.subscription().map(|x| WindowMessage::UDSScanner(x)))
            }
            return Subscription::batch(batch)
        }
        Subscription::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        // If not in launcher mode we should draw the bottom status bar as well
        return if let WindowState::Launcher { .. } = self.state {
            self.state.view().into()
        } else {
            // Draw the status bar!
            let t = match self.server.as_ref().unwrap().is_connected() {
                true => Text::new("Connected").color(Color::from_rgb8(0, 128, 0)),
                false => Text::new("Disconnected").color(Color::from_rgb8(128, 0, 0))
            };

            let v = Text::new(format!("{}V", self.voltage)).color(
                if self.voltage > 11.0 { // Low battery alert threshold
                    Color::from_rgb8(0, 128, 0)
                } else {
                    Color::from_rgb8(128, 0, 0)
                }
            );
            let page_name = &self.state.get_name();
            let view_contents = self.state.view();
            let mut s_bar = Row::new().padding(5).spacing(5).height(Length::Units(40))
                    .push(Row::new().spacing(5)
                    .push(Text::new("Status: "))
                    .push(t)
                )
                .push(Space::with_width(Length::Units(50)))
                .push(Row::new()
                    .spacing(5)
                    .push(Text::new("Battery voltage: "))
                    .push(v)
                ).push(Space::with_width(Length::Fill)
                );
            if page_name != &WindowStateName::Home {
                s_bar = s_bar.push(
                Row::new()
                    .spacing(5)
                    .push(Button::new(&mut self.back_btn_state,Text::new("Go home"))
                        .on_press(WindowMessage::GoHome)
                    )
                )
            }
            let c = Column::new()
                .push(view_contents)
                .push(Space::with_height(Length::Fill))
                .push(Rule::horizontal(2))
                .push(s_bar);
            c.height(Length::Fill).width(Length::Fill).into()
            //Container::new(c).center_x().center_y().into()
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
