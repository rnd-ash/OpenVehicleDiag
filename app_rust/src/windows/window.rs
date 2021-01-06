use std::fmt::{format, Debug};
use crate::{commapi, passthru::*};
use iced::{button, executor, pick_list, Align, Application, Button, Column, Command, Container, Element, Length, PickList, Row, Settings, Text, Sandbox, Radio, Subscription};
use J2534Common::Loggable;
use crate::commapi::passthru_api::PassthruApi;
use crate::commapi::comm_api::{ComServer, FilterType, ComServerError};
use crate::windows::window::ApplicationError::DriverError;
use serde::export::Formatter;
use crate::windows::window::LauncherMessage::LaunchRequested;
use crate::windows::launcher::{Launcher, LauncherMessage};
use crate::windows::home::{Home, HomeMessage};

#[derive(Debug, Clone)]
pub (crate) enum ApplicationError {
    DriverError(ComServerError)
}

#[derive(Debug, Clone)]
pub enum WindowState {
    Launcher { launcher: Launcher },
    Home {home: Home, api: String},
    CanTracer {

    },
}

impl<'a> WindowState {
    fn view(&mut self) -> Element<WindowMessage> {
        match self {
            Self::Launcher { launcher } => launcher.view().map(|x| WindowMessage::Launcher(x)).into(),
            Self::Home { home, .. } => home.view().map(|x| WindowMessage::Home(x)).into(),
            _ => unimplemented!()
        }
    }

    fn update(&mut self, msg: WindowMessage) -> Option<Box<dyn ComServer>> {
        match self {
            Self::Launcher { launcher } => {
                if let WindowMessage::Launcher(x) = msg {
                    return launcher.update(x);
                }
            },
            Self::Home {home, .. } => {
                if let WindowMessage::Home(x) = msg {
                    home.update(x);
                }
            }
            _ => unimplemented!()
        }
        None
    }
}



#[derive(Debug, Clone)]
pub enum WindowMessage {
    Launcher(LauncherMessage),
    Home(HomeMessage)
}


pub struct MainWindow {
    state: WindowState
}

impl Application for MainWindow {
    type Executor = executor::Default;
    type Message = WindowMessage;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {
            state: WindowState::Launcher { launcher : Launcher::new() }
        }, Command::none())
    }

    fn title(&self) -> String {
        match &self.state {
            WindowState::Launcher { .. } => "OpenVehicleDiag launcher".into(),
            WindowState::Home { api, .. } => format!("OpenVehicleDiag ({} mode)", api),
            WindowState::CanTracer { .. } => "OpenVehicleDiag CanTracer".into()
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<WindowMessage> {
        if let Some(srv) = self.state.update(message) {
            let api = srv.get_api().into();
            self.state = WindowState::Home {
                home: Home::new(srv),
                api
            }
        }
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match &self.state {
            WindowState::Home {home, ..} => {
                home.subscribe().map(WindowMessage::Home)
            }
            _ => Subscription::none()
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.state.view().into()
    }
}
