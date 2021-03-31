use crate::{commapi::{comm_api::{Capability, ComServer, ISO15765Config}, protocols::{ProtocolServer, obd2::{self, ObdServer, service09::Service09Data}}}, themes::button_coloured};
use crate::commapi::protocols::vin::Vin;
use crate::themes::{button_outlined, text, title_text, ButtonType, TextType, TitleSize};
use iced::{button, Align, Button, Column, Element, Length, Row, Space, Text};

#[derive(Debug, Clone)]
pub enum OBDMessage {
    InitOBD_IsoTP,
    Disconnect,
    ChooseService(u8),
}

#[derive(Debug, Clone)]
pub struct OBDHome {
    server: Box<dyn ComServer>,
    kline_state: button::State,
    can_state: button::State,
    obd_server: Option<ObdServer>,
    in_session: bool,
    s09_data: Service09Data,
    curr_service: u8,
    service_btn_states: [button::State; 10]
}

impl OBDHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            kline_state: Default::default(),
            can_state: Default::default(),
            obd_server: None,
            in_session: false,
            s09_data: Default::default(),
            curr_service: 0,
            service_btn_states: [button::State::default(); 10]
        }
    }

    pub fn update(&mut self, msg: &OBDMessage) -> Option<OBDMessage> {
        match msg {

            OBDMessage::InitOBD_IsoTP => {
                // Try all the CAN IDs
                for test_id in [0x07E8, 0x07E9, 0x07E0].iter() {
                    let cfg = ISO15765Config {
                        baud: 500_000,
                        send_id: 0x07DF,
                        recv_id: *test_id,
                        block_size: 8,
                        sep_time: 20,
    
                    };
                    if let Ok(server) = ObdServer::start_diag_session(self.server.clone(), &cfg, None) {
                        if let Ok(r) = server.req_service09(|x| Ok(x.get_everything(&server))) {
                            self.s09_data = r;
                        }
                        self.obd_server = Some(server);
                        self.in_session = true;
                        self.curr_service = 0; // Reset to landing page of OBD
                        println!("Found OBD receiver on address 0x{:04X}", test_id);
                        break;
                    }
                }
            },
            OBDMessage::Disconnect => {
                if self.obd_server.is_some() {
                    self.obd_server.take(); // Take and destroy
                    self.in_session = false;
                }
            },
            &OBDMessage::ChooseService(sid) => {
                self.curr_service = sid; // What service UI should we be in?
            }
        }
        None
    }

    pub fn view(&mut self) -> Element<OBDMessage> {
        if self.in_session {
            match self.curr_service {
                0x09 => self.create_s09_ui(),
                _ => self.create_main_ui()
            }
        } else {
            self.create_connect_ui()
        }
    }

    pub fn create_main_ui(&mut self) -> Element<OBDMessage> {
        
        let mut row = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center);

        let mut support_list = self.obd_server.as_ref().unwrap().get_supported_services();
        support_list.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap());

        for (idx, state) in self.service_btn_states.iter_mut().enumerate() {
            let (supported, pos, name) = &support_list[idx];
            let mut btn = button_outlined(state, &name, ButtonType::Info);
            if supported == &true {
                btn = btn.on_press(OBDMessage::ChooseService(*pos as u8))
            }
            row = row.push(btn)
        }


        Column::new()
        .padding(10)
        .spacing(10)
        .push(title_text("OBD Diagnostics", TitleSize::P2))
        .push(button_outlined(&mut self.can_state, "Disconnect", ButtonType::Primary).on_press(OBDMessage::Disconnect))
        .push(row)   
        
        .into()
    }


    pub fn create_connect_ui(&mut self) -> Element<OBDMessage> {
        let obd_btn = button_outlined(
            &mut self.kline_state,
            "K-Line not implemented",
            ButtonType::Danger,
        ); // TODO Add K-LINE OBD
        let can_btn = match self.server.get_capabilities().supports_iso15765() {
            Capability::Yes => {
                button_outlined(&mut self.can_state, "OBD over CANBUS", ButtonType::Danger)
                    .on_press(OBDMessage::InitOBD_IsoTP)
            }
            _ => Button::new(
                &mut self.can_state,
                Text::new("No CANBUS Support on adapter"),
            ),
        };

        let mut btn_row = Row::new()
        .padding(10)
        .spacing(10);

        let mut connect_shown = false;
        if self.server.get_capabilities().supports_iso9141() == Capability::Yes {
            btn_row = btn_row.push(obd_btn);
            connect_shown = true;
        }
        if self.server.get_capabilities().supports_iso15765() == Capability::Yes {
            btn_row = btn_row.push(can_btn);
            connect_shown = true;
        }

        // No way to talk to OBD!!
        if !connect_shown {
            return Column::new()
            .padding(10)
            .spacing(10)
            .push(title_text("OBD Diagnostics", TitleSize::P2))
            .push(text("Unfortunately, your adapter does not support ISO9141 or ISO15765.", TextType::Warning))
            .push(btn_row)
            .align_items(Align::Center)
            .into()
        }


        Column::new()
            .padding(10)
            .spacing(10)
            .push(title_text("OBD Diagnostics", TitleSize::P2))
            .push(Space::with_height(Length::Units(10)))
            .push(btn_row)
            .align_items(Align::Center)
            .into()
    }

    pub fn create_s09_ui(&mut self) -> Element<OBDMessage> {
        Column::new()
            .push(title_text("Vehicle information", TitleSize::P3))
            .push(text(format!("VIN: {}", self.s09_data.vin).as_str(), TextType::Normal))
            .push(text(format!("ECU Name: {}", self.s09_data.ecu_name).as_str(), TextType::Normal))
            .push(text(format!("Calibration ID: {}", self.s09_data.calibration_id).as_str(), TextType::Normal))
            .push(text(format!("CVNs: {:?}", self.s09_data.cvns).as_str(), TextType::Normal))
            .push(self.add_back_button())
            .into()

    }

    pub fn add_back_button(&mut self) -> Element<OBDMessage> {
        button_coloured(&mut self.service_btn_states[0], "Go back", ButtonType::Primary).on_press(OBDMessage::ChooseService(0)).into()
    }
}
