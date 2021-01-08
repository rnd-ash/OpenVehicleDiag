use crate::commapi::comm_api::{ComServer, Capability, CanFrame, FilterType};
use iced::{Element, Column, Text, Align, Container, Length, Subscription, Row, Checkbox, Rule, Color, Space, button, ProgressBar};
use iced::time;
use std::sync::Arc;
use std::time::Instant;
use iced::widget::checkbox::Style;
use crate::windows::window::WindowMessage;
use iced::widget::button::State;
use crate::windows::home::HomeMessage;
use std::fs::FileType;
use std::collections::HashMap;
use iced::widget::pane_grid::TitleBar;

#[derive(Debug, Clone)]
pub enum UDSHomeMessage {
    LaunchManual,
    LaunchAutomatic,
    NextMode,
    PrevMode,
    GoHome,
    ScanNextCID(Instant),
    Wait(Instant),
    Listen(Instant),
}

#[derive(Debug, Clone, Copy)]
struct IsoTpResp {
    id: u32,
    bs: u8,
    st: u8
}

#[derive(Debug, Clone)]
pub struct UDSHome {
    server: Box<dyn ComServer>,
    auto_state: button::State,
    manual_state: button::State,
    in_home: bool,
    auto_mode: bool,
    scan_stage: usize,
    curr_cid: u32,
    ignore_ids: HashMap<u32, CanFrame>,
    listen_duration_ms: u128,
    auto_found_ids: Vec<(u32, IsoTpResp)>
}

impl<'a> UDSHome {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            auto_state: button::State::default(),
            manual_state: button::State::default(),
            in_home: true,
            auto_mode: false,
            scan_stage: 0,
            curr_cid: 0,
            ignore_ids: HashMap::new(),
            auto_found_ids: Vec::new(),
            listen_duration_ms: 0
        }
    }

    pub fn get_next_cid(&mut self) {
        loop {
            self.curr_cid += 1;
            if self.ignore_ids.get(&self.curr_cid).is_none() { break }
        }
    }


    pub fn update(&mut self, msg: UDSHomeMessage) -> Option<UDSHomeMessage> {
        match msg {
            UDSHomeMessage::LaunchManual => {
                self.auto_mode = false;
                self.in_home = false;
                self.scan_stage = 0;
            }
            UDSHomeMessage::LaunchAutomatic => {
                if let Ok(_) = self.server.open_can_interface(500_000, false) {
                    let test_frame = CanFrame::new(0x07DF, &[0x02, 0x09, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);
                    if let Ok(_) = self.server.send_can_packets(&[test_frame], 100) {
                        self.auto_mode = true;
                        self.scan_stage = 0;
                        self.in_home = false;
                        return None
                    }
                }
            },
            UDSHomeMessage::NextMode => {
                self.scan_stage += 1;
            },
            UDSHomeMessage::PrevMode => {
                self.scan_stage -= 1;
            },
            UDSHomeMessage::GoHome => {
                self.in_home = true;
            },
            UDSHomeMessage::ScanNextCID(u) => {
                if self.curr_cid > 0x7FF { // Done!
                    self.scan_stage += 1;
                    // Close the can channel , it is no longer needed
                    self.server.close_can_interface();
                    return None
                }
                // Filter should be already open
                if let Ok(msgs) = self.server.read_can_packets(0, 1000) {
                    for candidate in msgs {
                        if candidate.dlc == 8 {
                            let data = candidate.get_data();
                            if data[0] == 0x30 { // Potential flow control
                                // Also its a frame we haven't seen before!
                                if self.ignore_ids.get(&candidate.id).is_none() {
                                    // Push the CAN ID that was sent previously and the locate response ID
                                    self.auto_found_ids.push((self.curr_cid-1, IsoTpResp { id: candidate.id, bs: data[1], st: data[2] }));
                                    break;
                                }
                            }
                        }
                    }
                }
                self.server.clear_can_rx_buffer(); // Clear any remaining packets in Rx buffer
                self.server.send_can_packets(&[CanFrame::new(self.curr_cid, &[0x10, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])], 0);
                self.get_next_cid();
            },
            UDSHomeMessage::Wait(u) => {
                self.scan_stage += 1;
                self.server.add_can_filter(FilterType::Pass, 0x0000, 0x000);
            }
            UDSHomeMessage::Listen(_) => {
                if self.listen_duration_ms >= 5000 {
                    self.scan_stage += 1;
                    // send first bogus CAN Packet
                    self.get_next_cid();
                    self.server.send_can_packets(&[CanFrame::new(self.curr_cid, &[0x10, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])], 0);
                    self.get_next_cid();
                    self.ignore_ids.insert(0x07DF, CanFrame::new(0x07DF, &[0x00])); // Add the ODB-II ID since we don't want to test it
                } else {
                    self.listen_duration_ms += 100;
                    if let Ok(msgs) = self.server.read_can_packets(0, 1000) {
                        for m in msgs {
                            self.ignore_ids.insert(m.id, m);
                        }
                    }
                }
            },
        }
        None
    }

    pub fn subscription(&self) -> Subscription<UDSHomeMessage> {
        if self.auto_mode && !self.in_home {
            match self.scan_stage {
                0 => time::every(std::time::Duration::from_millis(4000)).map(UDSHomeMessage::Wait),
                1 => time::every(std::time::Duration::from_millis(100)).map(UDSHomeMessage::Listen),
                2 => time::every(std::time::Duration::from_millis(100)).map(UDSHomeMessage::ScanNextCID),
                _ => Subscription::none()
            }
        } else {
            Subscription::none()
        }
    }

    pub fn draw_home(&mut self) -> Element<UDSHomeMessage> {
        Row::new()
            .push(Space::with_width(Length::FillPortion(1)))
            .push(
                Column::new()
                    .spacing(5)
                    .padding(5)
                    .align_items(Align::Center)
                    .push(Text::new("Welcome to the UDS Diagnostics page"))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("There are 2 modes of operation for this tool, please select wisely!"))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("Automatic").size(20))
                    .push(Text::new(
                        "In this mode, OpenVehicleDiag will attempt to locate \
                all diagnosable ECU's in your vehicle, and will then determine which \
                UDS commands each ECU supports"
                    ))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("Manual").size(20))
                    .push(Text::new(
                        "In this mode, you have to know the Send and Receive ID for your \
                ECU's, as well as ISO-TP configuration settings. Then you can manually \
                interrogate each ECU with custom ISO-TP commands"
                    ))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("If you don't know what any of this means, please select Automatic"))
                    .push(Row::new()
                        .align_items(Align::Center)
                        .push(button::Button::new(&mut self.auto_state, Text::new("Automatic")).on_press(UDSHomeMessage::LaunchAutomatic))
                        .push(Space::with_width(Length::Fill))
                        .push( button::Button::new(&mut self.manual_state, Text::new("Manual")).on_press(UDSHomeMessage::LaunchManual))
                    ).width(Length::FillPortion(1)))
            .push(Space::with_width(Length::FillPortion(1)))
            .into()
    }

    pub fn draw_scan_stage(&mut self) -> Element<UDSHomeMessage> {
        match self.auto_mode {
            true => self.draw_auto_stage(),
            false => self.draw_manual_stage()
        }
    }

    pub fn view(&mut self) -> Element<UDSHomeMessage> {
       if self.in_home {
           self.draw_home()
       } else {
           self.draw_scan_stage()
       }
    }

    pub fn draw_auto_stage(&mut self) -> Element<UDSHomeMessage> {
        match self.scan_stage {
            0 => {
                Column::new()
                    .push(Text::new("Please wait..."))
                    .into()
            },
            1 => {
                Column::new()
                    .push(Text::new("Listening for existing CAN Traffic"))
                    .push(ProgressBar::new((0.0 as f32)..=(5000.0 as f32), self.listen_duration_ms as f32))
                    .push(Text::new(format!("Listened to {} CAN Messages", self.ignore_ids.len())))
                    .into()
            },
            2 | 3 => {
                let mut c = Column::new()
                    .push(Text::new(format!("Testing CID 0x{:04X}", self.curr_cid)))
                    .push(ProgressBar::new((0.0 as f32)..=(0x07FF as f32), self.curr_cid as f32))
                    .push(Text::new("CIDs found"));

                for (x, y) in &self.auto_found_ids {
                    c = c.push(Text::new(format!("Found request ID 0x{:04X}, Control found with ID 0x{:04X}. ECU asked for a block size of {} and a separation time of {}ms", x, y.id, y.bs, y.st)))
                }
                c.into()
            }
            _ => unimplemented!()
        }
    }

    pub fn draw_manual_stage(&mut self) -> Element<UDSHomeMessage> {
        unimplemented!()
    }
}