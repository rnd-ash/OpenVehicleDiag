use iced::{executor, Application, Column, Command, Text};

use crate::passthru::{PassthruDevice, PassthruDrv};

use super::launcher::{self, Launcher};

enum AppWindow {
    Launcher {},
    Home {},
}
