use crate::commapi::comm_api::{ComServer, CanFrame, FilterType, ISO15765Config};
use iced::{Element, Column, Text, Align, Length, Subscription, Row, Space, button, ProgressBar};
use std::time::Instant;
use std::fs::{File};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::commapi::protocols::uds::{UDSCommand, UDSRequest, UDSResponse, UDSProcessError};
use std::io::{Write, Read};
use crate::themes::{text, TextType, progress_bar, ButtonType, title_text, button_outlined, TitleSize};

use super::uds_manual::{UDSManual, UDSManualMessage};

#[derive(Debug, Clone)]
pub enum UDSHomeMessage {
    LaunchManual,
    LaunchAutomatic,
    NextMode,
    PrevMode,
    GoHome,
    ScanNextCID,
    Wait(Instant),
    Listen(Instant),
    InterrogateECU,
    OpenFile,
    ManualMessage(UDSManualMessage),
    SaveScanResults
}

const MAX_CID_STD: u32 = 0x07FF; // 11bit ID
const MAX_CID_EXT: u32 = 0x1FFFFFFF; // 29bit - Damn this scan will take forever!

const WAIT_MS: u128 = 2000;
const LISTEN_MS: u128 = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CarECUs {
    vehicle_name: String,
    vehicle_year: u32,
    vehicle_oem: String,
    ecus: Vec<ECUISOTPSettings>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub (crate) struct ECUISOTPSettings {
    pub name: String,
    pub send_id: u32,
    pub flow_control_id: u32,
    pub block_size: u32,
    pub sep_time_ms: u32,
    pub supports_uds: Option<bool>,
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
    auto_found_ids: Vec<(u32, ECUISOTPSettings)>,
    auto_scan_result_ids: Vec<(u32, bool)>,
    save_text: String,
    manual_ui: Option<UDSManual>
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
            curr_ecu_idx: 0,
            manual_ui: None,
            save_text: "".into()
        }
    }

    pub fn get_next_cid(&mut self) {
        loop {
            self.curr_cid += 1;
            if self.ignore_ids.get(&self.curr_cid).is_none() { break }
        }
    }

    pub fn interrogate_ecu(&mut self, req: UDSRequest, ecu_idx: usize) -> Result<UDSResponse, UDSProcessError> {
        println!("Interrogate ecu {}", ecu_idx);
        let comm_data: (u32, ECUISOTPSettings) = self.auto_found_ids[ecu_idx].clone();
        let comm_settings = ISO15765Config {
            send_id: comm_data.0,
            recv_id: comm_data.1.flow_control_id,
            block_size: comm_data.1.block_size,
            sep_time: comm_data.1.sep_time_ms
        };
        let x = req.run_cmd_can(&mut self.server, &comm_settings);
        match &x {
            Ok(e) => println!("OK: {:?}", e),
            Err(e) => println!("ERR: {:?}", e)
        };
        x
    }

    pub fn update(&mut self, msg: &UDSHomeMessage) -> Option<UDSHomeMessage> {
        match msg {
            UDSHomeMessage::LaunchManual => {
                self.auto_mode = false;
                self.in_home = false;
                self.scan_stage = 0;
            }
            UDSHomeMessage::LaunchAutomatic => {
                if self.server.open_can_interface(500_000, false).is_ok() {
                    let test_frame = CanFrame::new(0x07DF, &[0x02, 0x09, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00]);
                    if self.server.send_can_packets(&[test_frame], 100).is_ok() {
                        self.auto_mode = true;
                        self.scan_stage = 0;
                        self.in_home = false;
                        return Some(UDSHomeMessage::Wait(Instant::now()))
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
            UDSHomeMessage::ManualMessage(msg) => {
                if let Some(ref mut x) = self.manual_ui {
                    return x.update(msg).map(UDSHomeMessage::ManualMessage)
                }
            }

            UDSHomeMessage::SaveScanResults => {
                if !self.auto_found_ids.is_empty() {
                    let time = chrono::Utc::now();
                    let path = std::env::current_dir().unwrap().join(format!("scan-{}.ovdjson", time.format("%F-%H_%M_%S")));
                    match File::create(&path) {
                        Ok(mut f) => {
                            let mut ecus :Vec<ECUISOTPSettings> = Vec::new();
                            for (_, res) in &self.auto_found_ids {
                                if res.supports_uds.is_some() {
                                    ecus.push(res.clone())
                                }
                            }
                            let car = CarECUs{
                                vehicle_name: "Unknown".into(),
                                vehicle_year: 2000,
                                vehicle_oem: "Unknown".into(),
                                ecus
                            };
                            self.save_text = match f.write_all(serde_json::to_string_pretty(&car).unwrap().as_bytes()) {
                                Ok(_) => format!("Scan results saved to {}", &path.as_os_str().to_str().unwrap()),
                                Err(e) => format!("Error writing results to {} - {}", &path.as_os_str().to_str().unwrap(), e)
                            };
                        },
                        Err(e) => {
                            self.save_text = format!("Error saving file - {}", e)
                        }
                    }
                }
            }
            UDSHomeMessage::OpenFile => {
                if let nfd::Response::Okay(f_path) = nfd::open_file_dialog(Some("ovdjson"), None).unwrap_or(nfd::Response::Cancel) {
                    if let Ok(mut file) = File::open(f_path) {
                        let mut str = "".into();
                        file.read_to_string(&mut str);

                        let parse : serde_json::Result<CarECUs>  = serde_json::from_str(&str);
                        match parse {
                            Ok(car) => {
                                println!("Creating manual mode");
                                self.manual_ui = Some(UDSManual::new(car.ecus, self.server.clone()))
                            },
                            Err(e) => {
                                eprintln!("Parsing of JSON failed {}", e);
                            }
                        }
                    }
                }
            }

            UDSHomeMessage::ScanNextCID => {
                if self.curr_cid >= MAX_CID_STD { // Done!
                    self.scan_stage = 3;
                    self.curr_ecu_idx = 0;
                    // Close the can channel , it is no longer needed
                    self.server.close_can_interface();
                    return None
                }
                // Filter should be already open
                let mut found = false;
                let clock = Instant::now();
                while clock.elapsed().as_millis() < 50 && !found {
                    if let Ok(msgs) = self.server.read_can_packets(0, 1000) {
                        for candidate in msgs {
                            if candidate.dlc == 8 {
                                let data = candidate.get_data();
                                if data[0] == 0x30 { // Potential flow control
                                    // Also its a frame we haven't seen before!
                                    if self.ignore_ids.get(&candidate.id).is_none() {
                                        // Push the CAN ID that was sent previously and the locate response ID
                                        self.auto_found_ids.push((self.curr_cid - 1, ECUISOTPSettings {
                                            name: "ECU name unknown".to_string(),
                                            send_id: self.curr_cid - 1,
                                            flow_control_id: candidate.id,
                                            block_size: data[1] as u32,
                                            sep_time_ms: data[2] as u32,
                                            supports_uds: None
                                        }));
                                        // Also, add the new ID to the ignore list so we don't scan on the Flow control ID
                                        self.ignore_ids.insert(candidate.id, candidate);
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5)); // Stops the machine from exploding
                }
                self.server.clear_can_rx_buffer(); // Clear any remaining packets in Rx buffer
                self.server.send_can_packets(&[CanFrame::new(self.curr_cid, &[0x10, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])], 0);
                self.get_next_cid();
                return Some(UDSHomeMessage::ScanNextCID)
            },
            UDSHomeMessage::Wait(elapsed) => {
                return if elapsed.elapsed().as_millis() > WAIT_MS { // Finished waiting...
                    self.listen_duration_ms = 0;
                    self.scan_stage += 1;
                    self.server.add_can_filter(FilterType::Pass, 0x0000, 0x000);
                    Some(UDSHomeMessage::Listen(Instant::now())) // Begin listening!
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    self.listen_duration_ms = elapsed.elapsed().as_millis();
                    Some(UDSHomeMessage::Wait(*elapsed)) // Continue to wait
                }
            }
            UDSHomeMessage::Listen(x) => {
                return if x.elapsed().as_millis() >= LISTEN_MS {
                    println!("Listen complete");
                    self.scan_stage += 1;
                    // send first bogus CAN Packet
                    self.curr_cid = 0x0000;
                    self.get_next_cid();
                    self.server.send_can_packets(&[CanFrame::new(self.curr_cid, &[0x10, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])], 0);
                    self.get_next_cid();
                    self.ignore_ids.insert(0x07DF, CanFrame::new(0x07DF, &[0x00])); // Add the OBD-II ID since we don't want to test it
                    Some(UDSHomeMessage::ScanNextCID)
                } else {
                    self.listen_duration_ms = x.elapsed().as_millis();
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    if let Ok(msgs) = self.server.read_can_packets(0, 1000) {
                        for m in msgs {
                            self.ignore_ids.insert(m.id, m);
                        }
                    }
                    Some(UDSHomeMessage::Listen(*x))
                }
            },
            UDSHomeMessage::InterrogateECU => {
                self.scan_stage = 4;
                println!("{}", self.curr_ecu_idx);
                if self.curr_ecu_idx as usize >= self.auto_found_ids.len() {
                    println!("Scan over");
                    self.scan_stage = 5;
                    return None;
                }
                let req = UDSRequest::new(UDSCommand::TesterPresent, &[0x01]);
                let res = self.interrogate_ecu(req, self.curr_ecu_idx as usize);
                match res {
                    Ok(_) => self.auto_found_ids[self.curr_ecu_idx as usize].1.supports_uds = Some(true),
                    Err(e) => match e {
                        UDSProcessError::InvalidCommand => self.auto_found_ids[self.curr_ecu_idx as usize].1.supports_uds = Some(false),
                        UDSProcessError::InvalidErrorCode => self.auto_found_ids[self.curr_ecu_idx as usize].1.supports_uds = Some(false),
                        UDSProcessError::InvalidDataLen => self.auto_found_ids[self.curr_ecu_idx as usize].1.supports_uds = Some(false),
                        _ => {}
                    }
                }
                self.curr_ecu_idx += 1;
                return Some(UDSHomeMessage::InterrogateECU)
            }
        }
        None
    }

    pub fn subscription(&self) -> Subscription<UDSHomeMessage> {
        Subscription::none()
    }

    pub fn draw_home(&mut self) -> Element<UDSHomeMessage> {
        Row::new()
            .push(Space::with_width(Length::FillPortion(1)))
            .push(
                Column::new()
                    .spacing(10)
                    .padding(10)
                    .align_items(Align::Center)
                    .push(title_text("Welcome to the UDS Diagnostics page", TitleSize::P2))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("This will attempt to detect all UDS/KWP2000 compatible ECUs in your car \
                        and then let you send custom UDS commands to the ECUs based on the result of the scan"))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("Scan mode").size(20))
                    .push(Text::new(
                        "In this mode, OpenVehicleDiag will attempt to locate \
                all diagnosable ECU's in your vehicle by injecting ISO-TP can frames, and will then if \
                the ECU also supports UDS or KWP2000 protocol for diagnostics"))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new(
                        "Once the scan is complete, you can save the results to a file, and load it \
                        later on to play with UDS commands on various ECUs in your vehicle"
                    ))
                    .push(Space::with_height(Length::Units(10)))
                    .push(Text::new("If you don't have a scan save ovdjson file, scan the car first"))
                    .push(Row::new()
                        .align_items(Align::Center)
                        .push(button_outlined(&mut self.auto_state, "Scan my car", ButtonType::Success).on_press(UDSHomeMessage::LaunchAutomatic))
                        .push(Space::with_width(Length::Fill))
                        .push( button_outlined(&mut self.manual_state, "Load scan save file", ButtonType::Warning).on_press(UDSHomeMessage::OpenFile))
                    ).width(Length::FillPortion(3)))
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
        if let Some(ref mut x) = self.manual_ui {
            x.view().map(UDSHomeMessage::ManualMessage)
        } else if self.in_home {
            self.draw_home()
        } else {
            self.draw_scan_stage()
        }
    }

    pub fn draw_auto_stage(&mut self) -> Element<UDSHomeMessage> {
        match self.scan_stage {
            0 => {
                Column::new()
                    .push(title_text("Waiting for network to settle", TitleSize::P2))
                    .push(progress_bar(0.0_f32..=(WAIT_MS as f32), self.listen_duration_ms as f32, ButtonType::Info))
                    .spacing(10)
                    .padding(10)
                    .into()
            },
            1 => {
                Column::new()
                    .push(title_text("Listening to existing CAN Traffic", TitleSize::P2))
                    .push(progress_bar(0.0_f32..=(LISTEN_MS as f32), self.listen_duration_ms as f32, ButtonType::Info))
                    .push(text(format!("Found {} CAN ID's", self.ignore_ids.len()).as_str(), TextType::Normal))
                    .spacing(10)
                    .padding(10)
                    .into()
            },
            2 => {
                let mut c = Column::new()
                    .push(title_text("Probing for ISO-TP capable ECUs", TitleSize::P2))
                    .push(Text::new(format!("Testing CID 0x{:04X}", self.curr_cid)))
                    .push(progress_bar(0.0_f32..=(0x07FF as f32), self.curr_cid as f32, ButtonType::Info))
                    .push(Text::new("CIDs found"));
                for (x, y) in &self.auto_found_ids {
                    c = c.push(Text::new(format!("Found request ID 0x{:04X}, Control found with ID 0x{:04X}. ECU asked for a block size of {} and a separation time of {}ms", x, y.flow_control_id, y.block_size, y.sep_time_ms)))
                }
                c.spacing(10)
                .padding(10)
                .into()
            }
            3 => {
                let mut c = Column::new()
                    .spacing(10)
                    .padding(10)
                    .push(title_text(format!("Scan completed, found {} possible ECUs", self.auto_found_ids.len()).as_str(), TitleSize::P2));
                for (x, y) in &self.auto_found_ids {
                    c = c.push(text(format!("Potential Send ID 0x{:04X}, FC found with ID 0x{:04X}, asked for block size of {} and a separation time of {}ms", x, y.flow_control_id, y.block_size, y.sep_time_ms).as_str(), TextType::Success))
                }
                c = c.push(button_outlined(&mut self.auto_state, "Begin UDS Interrogation", ButtonType::Secondary).on_press(UDSHomeMessage::InterrogateECU));
                c.into()
            },
            4 => {
                let curr_ecu_id = if self.curr_ecu_idx as usize >= self.auto_found_ids.len() { self.auto_found_ids.len() - 1 } else { self.curr_ecu_idx as usize };
                Column::new()
                    .push(Text::new(format!("Scanning ECU  {} possible ECUs...Currently scanning 0x{:04X}", self.auto_found_ids.len(), self.auto_found_ids[curr_ecu_id].0)))
                    .push(ProgressBar::new(0.0_f32..=(self.auto_found_ids.len() as f32), self.curr_ecu_idx as f32))
                    .push(Space::with_height(Length::Units(10)))
                    .into()
            },
            5 => {
                let mut c = Column::new()
                    .spacing(10)
                    .padding(10)
                    .push(title_text("Scan results", TitleSize::P2))
                    .push(Space::with_height(Length::Units(10)));
                for (x, y) in &self.auto_found_ids {
                    c = c.push(match y.supports_uds {
                        None => text(format!("CAN ID 0x{:04X}: Could not communicate - False positive", x).as_str(), TextType::Danger),
                        Some(t) => if t {
                            text(format!("CAN ID 0x{:04X}: UDS/KWP2000 is supported!", x).as_str(), TextType::Success)
                        } else {
                            text(format!("CAN ID 0x{:04X}: UDS/KWP2000 is not supported - but ECU supports ISO-TP", x).as_str(), TextType::Warning)
                        }
                    });
                }
                c = c.push(Space::with_height(Length::Units(5)));
                c = c.push(button_outlined(&mut self.auto_state, "Save results to file", ButtonType::Secondary).on_press(UDSHomeMessage::SaveScanResults));
                c.push(Text::new(&self.save_text))
                    .into()
            }
            _ => unimplemented!()
        }
    }

    pub fn draw_manual_stage(&mut self) -> Element<UDSHomeMessage> {
        unimplemented!()
    }
}