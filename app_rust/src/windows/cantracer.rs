use crate::{commapi::{comm_api::{ComServer, FilterType}, iface::{CanbusInterface, IFACE_CFG, Interface, InterfaceConfig, InterfacePayload}}, themes::{TextType, checkbox, picklist, text}};
use crate::themes::{button_coloured, ButtonType};
use crate::windows::window::WindowMessage;
use iced::{pick_list, time};
use iced::{button, Color, Column, Element, Length, Row, Scrollable, Subscription, Text};
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum TracerMessage {
    NewData(Instant),
    ToggleCan,
    ToggleExt(bool),
    SelectBaud(CanSpeed),
    ToggleBinaryMode(bool),
}

#[derive(Debug, Clone)]
pub struct CanTracer {
    can_spd_state: pick_list::State<CanSpeed>,
    can_spd: CanSpeed,
    can_interface: CanbusInterface,
    btn_state: button::State,
    can_queue: HashMap<u32, InterfacePayload>,
    can_prev: HashMap<u32, InterfacePayload>,
    is_connected: bool,
    is_binary_fmt: bool,
    use_ext_can: bool,
    status_text: String,
    scroll_state: iced::scrollable::State,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanSpeed {
    pub baud: u32,
    text: &'static str
}

impl std::fmt::Display for CanSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.text))
    }
}

const CAN_SPEEDS: &[CanSpeed] = &[
    CanSpeed {baud: 5000, text: "5 kbit/s" },
    CanSpeed {baud: 10000, text: "10 kbit/s" },
    CanSpeed {baud: 20000, text: "20 kbit/s" },
    CanSpeed {baud: 25000, text: "25 kbit/s" },
    CanSpeed {baud: 33333, text: "33.3 kbit/s" },
    CanSpeed {baud: 50000, text: "50 kbit/s" },
    CanSpeed {baud: 80000, text: "80 kbit/s" },
    CanSpeed {baud: 83333, text: "83.3 kbit/s" },
    CanSpeed {baud: 100000, text: "100 kbit/s" },
    CanSpeed {baud: 125000, text: "125 kbit/s" },
    CanSpeed {baud: 200000, text: "200 kbit/s" },
    CanSpeed {baud: 250000, text: "250 kbit/s" },
    CanSpeed {baud: 500000, text: "500 kbit/s" },
    CanSpeed {baud: 1000000, text: "1 mbit/s" }
];

impl<'a> CanTracer {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            can_spd_state: Default::default(),
            can_spd: CAN_SPEEDS.iter().find(|x| x.baud == 500000).unwrap().clone(), // 500kbps
            can_interface: CanbusInterface::new_raw(server),
            btn_state: Default::default(),
            can_queue: HashMap::new(),
            can_prev: HashMap::new(),
            is_connected: false,
            is_binary_fmt: false,
            use_ext_can: false,
            status_text: "".into(),
            scroll_state: Default::default(),
        }
    }

    pub fn insert_frames_to_map(&mut self, frames: Vec<InterfacePayload>) {
        for f in frames {
            self.can_queue.insert(f.id, f);
        }
    }

    fn close_can(&mut self) {
        if let Err(e) = self.can_interface.close() {
            self.status_text = format!("Error closing CAN Interface {}", e)
        } else {
            self.is_connected = false;
            self.can_queue.clear();
        }
    }

    fn open_can(&mut self) {
        if let Err(e) = {
            let mut cfg = InterfaceConfig::new();
            cfg.add_param(IFACE_CFG::BAUDRATE, self.can_spd.baud);
            cfg.add_param(IFACE_CFG::EXT_CAN_ADDR, self.use_ext_can as u32);
            self.can_interface.setup(&cfg)
        }{
            self.status_text = format!("Error opening CAN Interface {}", e)
        } else {
            self.is_connected = true;
            if let Err(e) =
                self.can_interface.add_filter(FilterType::Pass{id: 0x0000, mask: 0x0000})
            {
                self.status_text = format!("Error setting CAN Filter {}", e)
            } else if let Err(e) = self.can_interface.send_data(
            &[InterfacePayload {
                    id: 0x07DF,
                    data: vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                    flags: vec![],
                }],
                0
            ) {
                self.status_text = format!("Error sending wake-up packet {}", e)
            }
        }
    }

    pub fn update(&mut self, msg: &TracerMessage) -> Option<WindowMessage> {
        match msg {
            TracerMessage::NewData(_) => {
                if let Ok(m) = self.can_interface.recv_data(100, 0) {
                    self.insert_frames_to_map(m)
                }
            }
            TracerMessage::ToggleCan => {
                if self.is_connected {
                    self.close_can();
                } else {
                    self.open_can();
                }
            }
            TracerMessage::ToggleBinaryMode(b) => self.is_binary_fmt = *b,
            TracerMessage::ToggleExt(use_ext) => {
                self.use_ext_can = *use_ext;
                // User is in session, reconnect
                if self.is_connected {
                    self.close_can();
                    self.open_can();
                }
            },
            TracerMessage::SelectBaud(b) => {
                self.can_spd = *b
            }
        }
        None
    }

    pub fn subscription(&self) -> Subscription<TracerMessage> {
        if self.is_connected {
            return time::every(std::time::Duration::from_millis(10)).map(TracerMessage::NewData);
        }
        Subscription::none()
    }

    pub fn view(&mut self) -> Element<TracerMessage> {
        let btn = match self.is_connected {
            false => button_coloured(&mut self.btn_state, "Connect", ButtonType::Info),
            true => button_coloured(&mut self.btn_state, "Disconnect", ButtonType::Info),
        }
        .on_press(TracerMessage::ToggleCan);

        let speed_selector = picklist(&mut self.can_spd_state, CAN_SPEEDS, Some(self.can_spd), TracerMessage::SelectBaud);

        let check = self.is_binary_fmt;

        let ext_toggle = checkbox(self.use_ext_can, "Use Extended CAN", TracerMessage::ToggleExt);

        let mut r = Row::new();
        if !self.is_connected {
            r = r.push(text("CAN Speed: ", TextType::Normal))
                .push(speed_selector)

        }

        Column::new()
            .padding(10)
            .spacing(10)
            .push(Text::new("CAN Tracer"))
            .push(r)
            .push(btn)
            .push(ext_toggle)
            .push(checkbox(
                check,
                "View CAN in Binary",
                TracerMessage::ToggleBinaryMode,
            ))
            .push(
                Scrollable::new(&mut self.scroll_state)
                    .height(Length::Fill)
                    .push(Self::build_can_list(
                        &self.is_binary_fmt,
                        &self.can_queue,
                        &mut self.can_prev,
                    )),
            )
            .into()
    }

    pub fn build_can_list(
        binary: &bool,
        curr_data: &HashMap<u32, InterfacePayload>,
        old_data: &mut HashMap<u32, InterfacePayload>,
    ) -> Element<'a, TracerMessage> {
        let mut col = Column::new();
        let mut x: Vec<u32> = curr_data.keys().into_iter().copied().collect();
        x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for cid in x {
            let i = curr_data.get(&cid).unwrap();
            let mut container = Row::new();
            container = container.push(
                Row::new()
                    .push(Text::new(format!("CID: {:04X}", i.id)))
                    .width(Length::Units(200)),
            );
            if let Some(old_frame) = old_data.get(&cid) {
                // Old frame exists, try to work out what changed
                let old_data = &old_frame.data;
                for (i, byte) in i.data.iter().enumerate() {
                    container =
                        if *byte == old_data[i] {
                            // Same as old data
                            match binary {
                                true => container
                                    .push(Row::new().push(Text::new(format!("{:08b}", byte)))), // Cram all binary bits together
                                false => container.push(Row::new().push(
                                    Text::new(format!("{:02X}", byte)).width(Length::Units(30)),
                                )),
                            }
                        } else {
                            // Different data at this index, colour the text red
                            match binary {
                                true => container.push(
                                    Row::new().push(
                                        Text::new(format!("{:08b}", byte))
                                            .color(Color::from_rgb8(192, 0, 0)),
                                    ),
                                ), // Cram all binary bits together
                                false => container.push(
                                    Row::new().push(
                                        Text::new(format!("{:02X}", byte))
                                            .color(Color::from_rgb8(192, 0, 0))
                                            .width(Length::Units(30)),
                                    ),
                                ),
                            }
                        }
                }
                col = col.push(container)
            } else {
                // New frame, just add it
                for byte in &i.data {
                    container = match binary {
                        true => container.push(Row::new().push(Text::new(format!("{:08b}", byte)))), // Cram all binary bits together
                        false => container.push(
                            Row::new()
                                .push(Text::new(format!("{:02X}", byte)).width(Length::Units(30))),
                        ),
                    }
                }
            }
            old_data.insert(cid, i.clone()); // Update the old table
        }
        col.into()
    }
}
