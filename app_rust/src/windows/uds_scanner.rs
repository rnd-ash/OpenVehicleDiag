use crate::commapi::comm_api::{ComServer, Capability, CanFrame, FilterType, ISO15765Data, ISO15765Config};
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
use std::ops::Index;
use crate::commapi::protocols::uds::{UDSCommand, UDSRequest, UDSResponse, UDSProcessError};

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
    InterrogateECU(Instant)
}

const MAX_CID_STD: u32 = 0x07FF; // 11bit ID
const MAX_CID_EXT: u32 = 0x1FFFFFFF; // 29bit - Damn this scan will take forever!

#[derive(Debug, Clone, Copy)]
struct IsoTpResp {
    fc_id: u32,
    bs: u8,
    st: u8,
    uds_support: Option<bool>
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
    interrogation_state: u32,
    curr_ecu_idx: u32,
    auto_found_ids: Vec<(u32, IsoTpResp)>,
    auto_scan_result_ids: Vec<(u32, String)>,
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
            auto_scan_result_ids: Vec::new(),
            listen_duration_ms: 0,
            interrogation_state: 0,
            curr_ecu_idx: 0
        }
    }

    pub fn get_next_cid(&mut self) {
        loop {
            self.curr_cid += 1;
            if self.ignore_ids.get(&self.curr_cid).is_none() { break }
        }
    }

    pub fn interrogateECU(&mut self, req: UDSRequest, ecu_idx: usize) -> Result<UDSResponse, UDSProcessError> {
        let comm_data: (u32, IsoTpResp) = self.auto_found_ids[ecu_idx].clone();
        let comm_settings = ISO15765Config {
            send_id: comm_data.0,
            recv_id: comm_data.1.fc_id,
            block_size: comm_data.1.bs as u32,
            sep_time: comm_data.1.st as u32
        };
        let x = req.run_cmd_can(&mut self.server, &comm_settings);
        match &x {
            Ok(e) => println!("OK: {:?}", e),
            Err(e) => println!("ERR: {:?}", e)
        };

        return x;
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
                if self.curr_cid >= MAX_CID_STD { // Done!
                    self.scan_stage = 3;
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
                                    self.auto_found_ids.push((self.curr_cid-1, IsoTpResp { fc_id: candidate.id, bs: data[1], st: data[2], uds_support: None }));
                                    // Also, add the new ID to the ignore list so we don't scan on the Flow control ID
                                    self.ignore_ids.insert(candidate.id, candidate);
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
                    self.curr_cid = 0x07D0;
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
            UDSHomeMessage::InterrogateECU(_) => {
                if self.curr_ecu_idx as usize == self.auto_found_ids.len()-1 {
                    self.scan_stage += 1
                }

                let ecu_id = self.auto_found_ids[self.curr_ecu_idx as usize].0;
                let req1 = UDSRequest::new(UDSCommand::TesterPresent, &[0x01]);
                match self.interrogateECU(req1, self.curr_ecu_idx as usize) {
                    Ok(_) => self.auto_scan_result_ids.push((ecu_id, "ECU Supports UDS!".into())),
                    Err(e) => {
                        match e {
                            UDSProcessError::CommError(ce) => self.auto_scan_result_ids.push((ecu_id, format!("Failed to scan - Protocol error: {}!", ce))),
                            UDSProcessError::NoResponse => self.auto_scan_result_ids.push((ecu_id, "ECU did not reply! - Probably doesn't support UDS".into())),
                            UDSProcessError::InvalidDataLen => self.auto_scan_result_ids.push((ecu_id, "ECU replied with an invalid data length - Probably doesn't support UDS".into())),
                            UDSProcessError::InvalidErrorCode => self.auto_scan_result_ids.push((ecu_id, "ECU replied with an error which was not recognised! - Probably doesn't support UDS".into())),
                            UDSProcessError::InvalidCommand => self.auto_scan_result_ids.push((ecu_id, "ECU did not reply with a valid CMD ID! - Probably doesn't support UDS".into())),
                        }
                    }
                }
                self.curr_ecu_idx += 1;
            }
        }
        None
    }

    pub fn subscription(&self) -> Subscription<UDSHomeMessage> {
        if self.auto_mode && !self.in_home {
            match self.scan_stage {
                0 => time::every(std::time::Duration::from_millis(4000)).map(UDSHomeMessage::Wait),
                1 => time::every(std::time::Duration::from_millis(100)).map(UDSHomeMessage::Listen),
                2 => time::every(std::time::Duration::from_millis(50)).map(UDSHomeMessage::ScanNextCID),
                3 => Subscription::none(), // Scan summary page
                4 => time::every(std::time::Duration::from_millis(1000)).map(UDSHomeMessage::InterrogateECU),
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
                    .push(Text::new("Please wait - Waiting for CAN to settle"))
                    .align_items(Align::Center)
                    .height(Length::Fill)
                    .into()
            },
            1 => {
                Column::new()
                    .push(Text::new("Listening for existing CAN Traffic"))
                    .push(ProgressBar::new((0.0 as f32)..=(5000.0 as f32), self.listen_duration_ms as f32))
                    .push(Text::new(format!("Found {} CAN ID's", self.ignore_ids.len())))
                    .into()
            },
            2 => {
                let mut c = Column::new()
                    .push(Text::new(format!("Testing CID 0x{:04X}", self.curr_cid)))
                    .push(ProgressBar::new((0.0 as f32)..=(0x07FF as f32), self.curr_cid as f32))
                    .push(Text::new("CIDs found"));
                for (x, y) in &self.auto_found_ids {
                    c = c.push(Text::new(format!("Found request ID 0x{:04X}, Control found with ID 0x{:04X}. ECU asked for a block size of {} and a separation time of {}ms", x, y.fc_id, y.bs, y.st)))
                }
                c.into()
            }
            3 => {
                let mut c = Column::new()
                    .push(Text::new(format!("Scan completed, found {} possible ECUs", self.auto_found_ids.len())));
                for (x, y) in &self.auto_found_ids {
                    c = c.push(Text::new(format!("Potential Send ID 0x{:04X}, FC found with ID 0x{:04X}, asked for block size of {} and a separation time of {}ms", x, y.fc_id, y.bs, y.st)))
                }
                c = c.push(button::Button::new(&mut self.auto_state, Text::new("Begin UDS Interrogation")).on_press(UDSHomeMessage::NextMode));
                c.into()
            },
            _ => {
                let curr_ecu_id = if self.curr_ecu_idx as usize >= self.auto_found_ids.len() { self.auto_found_ids.len() - 1 } else { self.curr_ecu_idx as usize };
                let mut c = Column::new()
                    .push(Text::new(format!("Scanning ECU  {} possible ECUs...Currently scanning 0x{:04X}", self.auto_found_ids.len(), self.auto_found_ids[curr_ecu_id].0)))
                    .push(ProgressBar::new((0.0 as f32)..=(self.auto_found_ids.len() as f32), self.curr_ecu_idx as f32))
                    .push(Space::with_height(Length::Units(10)));
                for (x, y) in &self.auto_scan_result_ids {
                    c = c.push(Text::new(format!("Result for ECU 0x{:04X} - {}", x, y)))
                }
                c.into()
            },
            //_ => unimplemented!()
        }
    }

    pub fn draw_manual_stage(&mut self) -> Element<UDSHomeMessage> {
        unimplemented!()
    }
}