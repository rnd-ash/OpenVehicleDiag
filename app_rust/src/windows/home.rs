use crate::commapi::comm_api::{ComServer, Capability};
use iced::{Element, Column, Text, Align, Length, Row, Rule, Space, button};
use crate::windows::window::WindowMessage;
use crate::themes::{title_text, text, TextType, button_outlined, ButtonType, TitleSize};

#[derive(Debug, Clone)]
pub enum HomeMessage {

}


#[derive(Debug, Clone)]
pub struct Home {
    server: Box<dyn ComServer>,
    can_state: button::State,
    uds_state: button::State,
    obd_state: button::State
}

impl Home {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        let mut ret = Self {
            server,
            can_state: button::State::default(),
            uds_state: button::State::default(),
            obd_state: button::State::default()
        };
        // To guarantee everything works as it should, home screen should have NO interfaces open
        if let Err(e) = ret.server.close_can_interface() {
            eprintln!("ERROR closing CAN Interface {}", e)
        }
        if let Err(e) = ret.server.close_iso15765_interface() {
            eprintln!("ERROR closing ISO-TP Interface {}", e)
        }
        ret
    }

    pub fn update(&mut self, _msg: &HomeMessage) -> Option<WindowMessage> {
        None
    }

    pub fn view(&mut self) -> Element<WindowMessage> {
        let cap = self.server.get_capabilities();
        let contents = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .push(title_text("Welcome to OpenVehicleDiag", TitleSize::P1))
        // Render contents of info panel
            .push(Rule::horizontal(8))
            .push(title_text("Adapter Info:", TitleSize::P3))
            .push(text(format!("Hardware API: {}", self.server.get_api()).as_str(), TextType::Normal))
            .push(text(format!("Hardware name: {} (FW Version {})", cap.get_name(), cap.get_device_fw_version()).as_str(), TextType::Normal))
            .push(text(format!("Hardware vendor: {}", cap.get_vendor()).as_str(), TextType::Normal))
            .push(text(format!("Library path: {} (Version {})", cap.get_lib_path(), cap.get_library_version()).as_str(), TextType::Normal))
            .push(title_text("Supported protocols", TitleSize::P3))
            .push(
            Row::new().spacing(5)
                    .push(
                Column::new()
                        .push(text("CAN", TextType::Normal))
                        .push(text("ISO-TP", TextType::Normal))
                        .push(text("ISO9141", TextType::Normal))
                        .push(text("ISO14230", TextType::Normal)))
                    .push(
                    Column::new()
                            .push(Home::gen_cap_contents(cap.support_can_fd()))
                            .push(Home::gen_cap_contents(cap.supports_iso15765()))
                            .push(Home::gen_cap_contents(cap.supports_iso9141()))
                            .push(Home::gen_cap_contents(cap.supports_iso14230())))
                .push(Space::with_width(Length::Units(50)))
                  .push(
                        Column::new()
                            .push(text("J1850PWM", TextType::Normal))
                            .push(text("J1850VPW", TextType::Normal))
                            .push(text("DoIP", TextType::Normal)))
                .push(
                    Column::new()
                        .push(Home::gen_cap_contents(cap.supports_j1850pwm()))
                        .push(Home::gen_cap_contents(cap.supports_j1850vpw()))
                        .push(Home::gen_cap_contents(cap.supports_doip())))
            ).push( Column::new()
            .align_items(Align::Center)
            .spacing(5)
            .push(Text::new("Tools"))
            .push(button_outlined(&mut self.can_state, "CAN Analyzer", ButtonType::Primary).on_press(WindowMessage::GoCanTracer))
            .push(button_outlined(&mut self.uds_state, "Diagnostic Scanner", ButtonType::Primary).on_press(WindowMessage::GoUDS))
            .push(button_outlined(&mut self.obd_state, "OBD Tools", ButtonType::Primary).on_press(WindowMessage::GoOBD))
            );
        contents.into()
    }
}

impl<'a> Home {
    fn gen_cap_contents(cap: Capability) -> Element<'a, WindowMessage> {
        match cap {
            Capability::Yes => text("Yes", TextType::Success),
            Capability::No => text("No", TextType::Danger),
            Capability::NA => text("N/A", TextType::Disabled),
        }.into()
    }
}