use crate::themes::{button_coloured, container, text, toggle_theme, ButtonType, TextType};
use crate::windows::cantracer::{CanTracer, TracerMessage};
use crate::windows::diag_home::DiagHomeMessage;
use crate::windows::home::{Home, HomeMessage};
use crate::windows::launcher::{Launcher, LauncherMessage};
use crate::windows::obd::{OBDHome, OBDMessage};
use crate::{
    commapi::comm_api::{Capability, ComServer, ComServerError},
    themes, WIN_HEIGHT,
};
use iced::{
    button, executor, time, Align, Application, Column, Command, Container, Element, Length, Row,
    Rule, Space, Subscription, Text,
};
use std::fmt::Debug;
use std::time::Instant;

use super::diag_home::DiagHome;

// This can be modified by diagnostic sessions in order to disable going
// home option in case a sensitive operation is in progress!
// True by default unless a diagnostic session requests it to be disabled
static mut show_home: bool = true;

pub fn enable_home() {
    unsafe { show_home = true }
}
pub fn disable_home() {
    unsafe { show_home = false }
}

fn get_home() -> bool {
    unsafe { show_home }
}

#[derive(Debug, Clone)]
pub(crate) enum ApplicationError {
    DriverError(ComServerError),
}

#[derive(Debug, Clone)]
pub enum WindowState {
    Launcher(Launcher),
    Home(Home),
    CanTracer(CanTracer),
    DiagHome(DiagHome),
    OBDTools(OBDHome),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WindowStateName {
    Launcher,
    Home,
    CanTracer,
    DiagHome,
    OBDTools,
}

impl<'a> WindowState {
    fn view(&mut self) -> Element<WindowMessage> {
        match self {
            Self::Launcher(launcher) => launcher.view().map(WindowMessage::Launcher),
            Self::Home(home) => home.view(),
            Self::CanTracer(tracer) => tracer.view().map(WindowMessage::CanTracer),
            Self::DiagHome(h) => h.view().map(WindowMessage::DiagHome),
            Self::OBDTools(h) => h.view().map(WindowMessage::OBDTools),
        }
    }

    fn update(&mut self, msg: &WindowMessage) -> Option<WindowMessage> {
        match self {
            Self::Launcher(launcher) => {
                if let WindowMessage::Launcher(x) = msg {
                    return launcher.update(x);
                }
            }
            Self::Home(home) => {
                if let WindowMessage::Home(x) = msg {
                    return home.update(x);
                }
            }
            Self::CanTracer(tracer) => {
                if let WindowMessage::CanTracer(x) = msg {
                    return tracer.update(x);
                }
            }
            Self::DiagHome(d) => {
                if let WindowMessage::DiagHome(x) = msg {
                    return d.update(x).map(WindowMessage::DiagHome);
                }
            }
            Self::OBDTools(o) => {
                if let WindowMessage::OBDTools(x) = msg {
                    return o.update(x).map(WindowMessage::OBDTools);
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
            WindowState::DiagHome { .. } => WindowStateName::DiagHome,
            WindowState::OBDTools { .. } => WindowStateName::OBDTools,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WindowMessage {
    Launcher(LauncherMessage),
    Home(HomeMessage),
    CanTracer(TracerMessage),
    DiagHome(DiagHomeMessage),
    OBDTools(OBDMessage),
    StartApp(Box<dyn ComServer>),
    StatusUpdate(Instant),
    GoHome,      // Goto home page
    GoCanTracer, // Goto Can Tracer page
    GoUDS,       // Goto UDS Scanner page
    GoOBD,       // Goto OBD Toolbox page
    ToggleTheme, // Toggle the theme
}

pub struct MainWindow {
    state: WindowState,
    server: Option<Box<dyn ComServer>>,
    voltage: f32,
    poll_voltage: bool,
    back_btn_state: button::State,
    theme_toggle: button::State,
}

impl Application for MainWindow {
    type Executor = executor::Default;
    type Message = WindowMessage;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                state: WindowState::Launcher(Launcher::new()),
                server: None,
                voltage: 0.0,
                poll_voltage: false,
                back_btn_state: button::State::default(),
                theme_toggle: button::State::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        match &self.state {
            WindowState::Launcher { .. } => "OpenVehicleDiag launcher".into(),
            WindowState::Home { .. } => format!(
                "OpenVehicleDiag ({} mode)",
                self.server
                    .as_ref()
                    .map(|s| s.get_api())
                    .unwrap_or("Unknown")
            ),
            WindowState::CanTracer { .. } => "OpenVehicleDiag CanTracer".into(),
            WindowState::DiagHome { .. } => "OpenVehicleDiag Diagnostics Scanner".into(),
            WindowState::OBDTools { .. } => "OpenVehicleDiag OBD Toolbox".into(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<WindowMessage> {
        match message {
            WindowMessage::StatusUpdate(_) => {
                // On request for battery voltage reading, try to read from the adapter, but it might timeout
                // if the driver is under heady IO load, so then use the current voltage reading
                self.voltage = self
                    .server
                    .as_ref()
                    .unwrap()
                    .read_battery_voltage()
                    .unwrap_or(self.voltage)
            }
            WindowMessage::GoHome => {
                self.state = WindowState::Home(Home::new(self.server.clone().unwrap()))
            }
            WindowMessage::GoCanTracer => {
                self.state = WindowState::CanTracer(CanTracer::new(self.server.clone().unwrap()))
            }
            WindowMessage::GoUDS => {
                self.state = WindowState::DiagHome(DiagHome::new(self.server.clone().unwrap()))
            }
            WindowMessage::GoOBD => {
                self.state = WindowState::OBDTools(OBDHome::new(self.server.clone().unwrap()))
            }
            WindowMessage::ToggleTheme => toggle_theme(),
            _ => return self.update_children(&message),
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        if let WindowState::Launcher { .. } = self.state {
            Subscription::none()
        } else {
            // Ask for battery every 2 seconds (If supported)
            let mut batch: Vec<Subscription<WindowMessage>> = vec![];
            if self.poll_voltage {
                batch.push(
                    time::every(std::time::Duration::from_secs(2)).map(WindowMessage::StatusUpdate),
                );
            }
            // See if either other pages request update
            if let WindowState::CanTracer(tracer) = &self.state {
                batch.push(tracer.subscription().map(WindowMessage::CanTracer))
            } else if let WindowState::DiagHome(d) = &self.state {
                batch.push(d.subscription().map(WindowMessage::DiagHome))
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

            let v = if self.poll_voltage {
                if self.voltage < 12.0 && self.voltage > 11.5 {
                    text(format!("{}V", self.voltage).as_str(), TextType::Warning)
                } else if self.voltage < 11.5 {
                    text(format!("{}V", self.voltage).as_str(), TextType::Danger)
                } else {
                    text(format!("{}V", self.voltage).as_str(), TextType::Success)
                }
            } else {
                text("Not supported", TextType::Disabled)
            };
            let page_name = &self.state.get_name();
            let view_contents = Container::new(self.state.view())
                .height(Length::Units(WIN_HEIGHT as u16 - 50))
                .width(Length::Fill);
            let mut s_bar = Row::new()
                .padding(5)
                .spacing(5)
                .height(Length::Shrink)
                .push(
                    Row::new()
                        .spacing(5)
                        .push(Text::new("Status: "))
                        .push(t)
                        .align_items(Align::Center),
                )
                .push(Space::with_width(Length::Units(50)))
                .push(
                    Row::new()
                        .spacing(5)
                        .push(Text::new("Battery voltage: "))
                        .push(v),
                )
                .push(Space::with_width(Length::Fill));

            let mut btn_row = Row::new().spacing(5).push(
                button_coloured(
                    &mut self.theme_toggle,
                    "Toggle theme",
                    ButtonType::Secondary,
                )
                .on_press(WindowMessage::ToggleTheme),
            );

            if page_name != &WindowStateName::Home {
                let mut home_btn =
                    button_coloured(&mut self.back_btn_state, "Go home", ButtonType::Warning);
                if get_home() {
                    home_btn = home_btn.on_press(WindowMessage::GoHome);
                }
                btn_row = btn_row.push(home_btn)
            }
            s_bar = s_bar.push(btn_row).height(Length::Units(50));

            let mut c: Element<_> = Column::new()
                .push(view_contents)
                .push(Rule::horizontal(1))
                .push(s_bar)
                .into();
            if themes::is_debug() {
                c = c.explain(iced::Color::BLACK);
            }
            container(c).height(Length::Fill).width(Length::Fill).into()
        };
    }
}

impl MainWindow {
    fn update_children(&mut self, message: &WindowMessage) -> Command<WindowMessage> {
        // Special case handling
        if let Some(state) = self.state.update(message) {
            match state {
                WindowMessage::StartApp(srv) => {
                    self.server = Some(srv.clone_box());
                    self.poll_voltage = srv.get_capabilities().battery_voltage == Capability::Yes;
                    if self.poll_voltage {
                        self.voltage = self
                            .server
                            .as_ref()
                            .unwrap()
                            .read_battery_voltage()
                            .unwrap_or(0.0);
                    } else {
                        self.voltage = 12.0; // This is to allow scans which measure battery to occur
                    }
                    self.state = WindowState::Home(Home::new(srv));
                    Command::none()
                }
                _ => Command::perform(async move { state.clone() }, |x| x),
            }
        } else {
            Command::none()
        }
    }
}

impl Drop for MainWindow {
    fn drop(&mut self) {
        if let Some(mut s) = self.server.take() {
            s.close_iso15765_interface()
                .expect("Error closing ISO15765");
            s.close_can_interface()
                .expect("Error closing can Interface");
            s.close_device().expect("Error closing device");
        }
    }
}
