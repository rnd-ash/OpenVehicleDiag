use iced::{executor, Application};

use crate::passthru::{PassthruDevice, PassthruDrv};

enum mainapp {
    Launcher,
    MainWindow,
}

/*
impl Application for mainapp {
    type Executor = executor::Default;

    type Message;

    type Flags;

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        todo!()
    }

    fn title(&self) -> String {
        todo!()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        todo!()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        todo!()
    }
}
*/
