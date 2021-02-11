use crate::commapi::comm_api::{ComServer, CanFrame, FilterType};
use iced::{Checkbox, Color, Column, Element, Length, Row, Scrollable, Subscription, Text, button};
use iced::time;
use std::time::Instant;
use crate::windows::window::WindowMessage;
use std::collections::HashMap;
use crate::themes::{button_coloured, ButtonType};

#[derive(Debug, Clone)]
pub enum TracerMessage {
    NewData(Instant),
    ToggleCan,
    ToggleBinaryMode(bool)
}


#[derive(Debug, Clone)]
pub struct CanTracer {
    server: Box<dyn ComServer>,
    btn_state: button::State,
    can_queue: HashMap<u32, CanFrame>,
    can_prev: HashMap<u32, CanFrame>,
    is_connected: bool,
    is_binary_fmt: bool,
    status_text: String,
    scroll_state: iced::scrollable::State,
}

impl<'a> CanTracer {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            btn_state: Default::default(),
            can_queue: HashMap::new(),
            can_prev: HashMap::new(),
            is_connected: false,
            is_binary_fmt: false,
            status_text: "".into(),
            scroll_state: Default::default(),
        }
    }

    pub fn insert_frames_to_map(&mut self, frames: Vec<CanFrame>) {
        for f in frames {
            self.can_queue.insert(f.id, f);
        }
    }

    pub fn update(&mut self, msg: &TracerMessage) -> Option<WindowMessage> {
        match msg {
            TracerMessage::NewData(_) => {
                if let Ok(m) = self.server.as_ref().read_can_packets(0, 100) {
                    self.insert_frames_to_map(m)
                }
            },
            TracerMessage::ToggleCan => {
                if self.is_connected {
                    if let Err(e) = self.server.as_mut().close_can_interface() {
                        self.status_text = format!("Error closing CAN Interface {}", e)
                    } else {
                        self.is_connected = false;
                        self.can_queue.clear();
                    }
                } else if let Err(e) = self.server.as_mut().open_can_interface(500_000, false) {
                        self.status_text = format!("Error opening CAN Interface {}",  e)
                } else {
                    self.is_connected = true;
                    if let Err(e) = self.server.as_mut().add_can_filter(FilterType::Pass, 0x0000, 0x0000) {
                        self.status_text = format!("Error setting CAN Filter {}",  e)
                    } else if let Err(e) = self.server.send_can_packets(&[CanFrame::new(0x07DF, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])], 0) {
                        self.status_text = format!("Error sending wake-up packet {}",  e)
                    }
                }
            }
            TracerMessage::ToggleBinaryMode(b) => {
                self.is_binary_fmt = *b
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
            false =>button_coloured(&mut self.btn_state, "Connect", ButtonType::Info),
            true => button_coloured(&mut self.btn_state, "Disconnect", ButtonType::Info)
        }.on_press(TracerMessage::ToggleCan);
        let check = self.is_binary_fmt;

       Column::new().padding(10).spacing(10)
           .push(Text::new("CAN Tracer"))
           .push(btn)
           .push(Checkbox::new(check, "View CAN in Binary", TracerMessage::ToggleBinaryMode))
           .push(
               Scrollable::new(&mut self.scroll_state).height(Length::Fill).push(
                Self::build_can_list(&self.is_binary_fmt, &self.can_queue, &mut self.can_prev))
            )
           .into()
    }

    pub fn build_can_list(binary: &bool, curr_data: &HashMap<u32, CanFrame>, old_data: &mut HashMap<u32, CanFrame>) -> Element<'a, TracerMessage> {
        let mut col = Column::new();
        let mut x : Vec<u32> = curr_data.keys().into_iter().copied().collect();
        x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for cid in x {
            let i = curr_data.get(&cid).unwrap();
            let mut container = Row::new();
            container = container.push(Row::new().push(Text::new(format!("CID: {:04X}", i.id))).width(Length::Units(100)));
            if let Some(old_frame) = old_data.get(&cid) {
                // Old frame exists, try to work out what changed
                let old_data = old_frame.get_data();
                for (i, byte) in i.get_data().iter().enumerate() {
                    container = if *byte == old_data[i] { // Same as old data
                        match binary {
                            true => container.push(Row::new().push(Text::new(format!("{:08b}", byte)))), // Cram all binary bits together
                            false => container.push(Row::new().push(Text::new(format!("{:02X}", byte)).width(Length::Units(30))))
                        }
                    } else { // Different data at this index, colour the text red
                        match binary {
                            true => container.push(Row::new().push(Text::new(format!("{:08b}", byte)).color(Color::from_rgb8(192, 0, 0)))), // Cram all binary bits together
                            false => container.push(Row::new().push(Text::new(format!("{:02X}", byte)).color(Color::from_rgb8(192, 0, 0)).width(Length::Units(30))))
                        }
                    }
                }
                col = col.push(container)
            } else {
                // New frame, just add it
                for byte in i.get_data() {
                    container = match binary {
                        true => container.push(Row::new().push(Text::new(format!("{:08b}", byte)))), // Cram all binary bits together
                        false => container.push(Row::new().push(Text::new(format!("{:02X}", byte)).width(Length::Units(30))))
                    }
                }
            }
            old_data.insert(cid, *i); // Update the old table

        }
        col.into()
    }
}