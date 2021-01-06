use std::fmt::{format, Debug};
use crate::{commapi, passthru::*};
use iced::{button, executor, pick_list, Align, Application, Button, Column, Command, Container, Element, Length, PickList, Row, Settings, Text, Sandbox, Radio};
use J2534Common::Loggable;
use crate::commapi::passthru_api::PassthruApi;
use crate::commapi::comm_api::{ComServer, FilterType, ComServerError};
use crate::windows::window::ApplicationError::DriverError;
use serde::export::Formatter;
use crate::windows::window::LauncherMessage::LaunchRequested;
use crate::windows::launcher::{Launcher, LauncherMessage};

#[derive(Debug, Clone)]
pub (crate) enum ApplicationError {
    DriverError(ComServerError)
}

#[derive(Clone)]
pub struct Home {
    server: Box<dyn ComServer>
}

impl Debug for Home {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Home")
            .finish()
    }
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
            _ => unimplemented!()
        }
    }

    fn update(&mut self, msg: WindowMessage) {
        match self {
            Self::Launcher { launcher } => {
                if let WindowMessage::Launcher(x) = msg {
                    launcher.update(x)
                }
            },
            _ => unimplemented!()
        }
    }
}



#[derive(Debug, Clone)]
pub enum WindowMessage {
    Launcher(LauncherMessage)

}


pub struct MainWindow {
    state: WindowState
}

impl Sandbox for MainWindow {
    type Message = WindowMessage;

    fn new() -> Self {
        Self {
            state: WindowState::Launcher { launcher : Launcher::new() }
        }
    }

    fn title(&self) -> String {
        match &self.state {
            WindowState::Launcher { .. } => "OpenVehicleDiag launcher".into(),
            WindowState::Home { api, .. } => format!("OpenVehicleDiag ({} mode)", api),
            WindowState::CanTracer { .. } => "OpenVehicleDiag CanTracer".into()
        }
    }

    fn update(&mut self, message: Self::Message) {
        self.state.update(message)
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.state.view().into()
    }
}
