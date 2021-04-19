use std::{collections::HashMap, fs::File, io::Write, time::Instant};

use commapi::{
    comm_api::ISO15765Config,
    protocols::{kwp2000::KWP2000ECU, uds::UDSECU, ProtocolServer},
};
use iced::{Align, Column, Element, Length, Row, Space};

use crate::{commapi::{self, comm_api::{CanFrame, ComServer}, protocols::kwp2000::read_ecu_identification::read_dcx_mmc_id}, themes::{
        button_coloured, button_outlined, progress_bar, text, title_text, ButtonType, TextType,
    }};

use super::diag_home::{ECUDiagSettings, VehicleECUList};

#[derive(Debug, Clone)]
pub enum DiagScannerMessage {
    IncrementStage,
    ScanPoll,
    SaveResults,
}

#[derive(Debug, Clone)]
pub struct DiagScanner {
    server: Box<dyn ComServer>,
    curr_stage: usize,
    btn: iced::button::State,
    status: String,
    filter_idx: u32,
    clock: Instant,
    can_traffic_id_list: HashMap<u32, bool>,
    curr_scan_id: u32,
    stage2_results: HashMap<u32, Vec<u32>>,
    stage3_results: Vec<ISO15765Config>,
    stage4_results: Vec<ECUDiagSettings>, // Also used by stage 5
    save_attempted: bool,
    save_path: String,
}

impl DiagScanner {
    pub(crate) fn new(server: Box<dyn ComServer>) -> Self {
        Self {
            server,
            curr_stage: 0,
            btn: Default::default(),
            status: String::new(),
            filter_idx: 0,
            clock: Instant::now(),
            curr_scan_id: 0,
            can_traffic_id_list: HashMap::new(),
            stage2_results: HashMap::new(),
            stage3_results: Vec::new(),
            stage4_results: Vec::new(),
            save_attempted: false,
            save_path: "".into(),
        }
    }

    fn increment_stage(&mut self) -> Option<DiagScannerMessage> {
        match self.curr_stage {
            0 => {
                if self.server.read_battery_voltage().unwrap_or(0.0) < 11.7 {
                    self.status = "Battery voltage too low / Could not read battery voltage".into();
                    return None;
                }
                // Try to setup CAN Iface with open filter
                if let Err(e) = self.server.open_can_interface(500_000, false) {
                    self.status = format!("Could open CAN Interface ({})", e)
                } else {
                    // Opening interface was OK
                    match self.server.add_can_filter(
                        commapi::comm_api::FilterType::Pass,
                        0x00000000,
                        0x00000000,
                    ) {
                        Ok(f_idx) => {
                            // Send the OBD-II get VIN request via CAN. This should wake up the OBD-II port's CAN Iface
                            // on most cars
                            if self
                                .server
                                .send_can_packets(&[CanFrame::new(0x07DF, &[0x09, 0x02])], 0)
                                .is_err()
                            {
                                self.status = "Could not send wake up test packet".into();
                                self.server
                                    .close_can_interface()
                                    .expect("What a terrible failure. Closing CAN Iface failed!?");
                            } else {
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                self.filter_idx = f_idx;
                                self.curr_stage += 1; // We can progress to the next stage!
                                self.can_traffic_id_list.clear();
                                self.clock = Instant::now(); // Begin polling clock
                                return Some(DiagScannerMessage::ScanPoll); // Begin the polling!
                            }
                        }
                        Err(e) => {
                            // STOP THE CAN INTERFACE
                            self.server
                                .close_can_interface()
                                .expect("What a terrible failure. Closing CAN Iface failed!?");
                            self.status = format!("Could not set CAN filter ({})", e)
                        }
                    }
                }
            }
            1 => {
                // Accumulate scan results here
                self.can_traffic_id_list.insert(0x07DF, false); // Add OBD-II CAN ID so we don't scan that
                self.curr_stage += 1; // We can progress to the next stage!
                self.curr_scan_id = 0x0; // Set start ID to 0
                self.server.clear_can_rx_buffer();
                return Some(DiagScannerMessage::ScanPoll); // Begin the CAN interrogation (Stage 1)
            }
            2 => {
                if let Err(e) = self.server.close_can_interface() {
                    self.status = format!("Error closing old CAN Interface!: {}", e.err_desc);
                    return None;
                }
                self.curr_stage += 1;
                self.curr_scan_id = 0; // First entry in our array
                if let Err(e) = self.server.open_can_interface(500_000, false) {
                    self.status = format!("Error opening new CAN Interface!: {}", e.err_desc);
                    return None;
                }
                return Some(DiagScannerMessage::ScanPoll); // Begin the CAN interrogation (Stage 2)
            }
            3 => {
                // network cool down
                if let Err(e) = self.server.close_can_interface() {
                    self.status = format!("Error closing old CAN Interface!: {}", e.err_desc);
                    return None;
                }
                self.curr_stage += 1;
                self.curr_scan_id = 0; // First entry in our array
                std::thread::sleep(std::time::Duration::from_millis(100)); // Wait for adapter to rest (For some reason this works)
                self.clock = Instant::now(); // Reset clock
                return Some(DiagScannerMessage::ScanPoll);
            }
            4 => {
                self.curr_stage += 1;
                self.curr_scan_id = 0; // First entry in our array
                                       // Move to stage 5 (KWP2000 scan)
                return Some(DiagScannerMessage::ScanPoll);
            }
            5 => {
                self.curr_stage += 1;
                self.curr_scan_id = 0; // First entry in our array
                                       // Move to stage 6 (UDS scan)
                return Some(DiagScannerMessage::ScanPoll);
            }
            6 => {
                // Done scan!
                self.curr_stage += 1; // End page
                return None;
            }
            _ => {}
        }
        None
    }

    fn get_next_canid(&mut self) {
        loop {
            self.curr_scan_id += 1;
            if self.can_traffic_id_list.get(&self.curr_scan_id).is_none() {
                return; // Found a free ID
            }
        }
    }

    #[allow(unused_must_use)]
    fn poll(&mut self) -> Option<DiagScannerMessage> {
        match self.curr_stage {
            0 => None,
            1 => {
                if self.clock.elapsed().as_millis() >= 10000 {
                    Some(DiagScannerMessage::IncrementStage)
                } else {
                    for frame in &self.server.read_can_packets(0, 10000).unwrap_or_default() {
                        self.can_traffic_id_list.insert(frame.id, true);
                    }
                    Some(DiagScannerMessage::ScanPoll) // Keep polling
                }
            }
            2 => {
                if self.curr_scan_id >= 0x7FF {
                    // Scanning complete
                    Some(DiagScannerMessage::IncrementStage)
                } else if self.clock.elapsed().as_millis() >= 100 {
                    // Timeout waiting for response
                    self.get_next_canid();
                    self.server.clear_can_rx_buffer();
                    // Send a fake ISO-TP first frame. Tell the potential ECU we are sending 16 bytes to it. If it uses ISO-TP, it'll send back a
                    // flow control message back to OVD
                    self.server.send_can_packets(
                        &[CanFrame::new(
                            self.curr_scan_id,
                            &[0x10, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                        )],
                        0,
                    );
                    // Reset the timer
                    self.clock = Instant::now();
                    Some(DiagScannerMessage::ScanPoll)
                } else {
                    // Keep scanning for response messages
                    for frame in &self.server.read_can_packets(0, 10000).unwrap_or_default() {
                        if self.can_traffic_id_list.get(&frame.id).is_none() {
                            // Its a new frame we haven't seen before!
                            let payload = frame.get_data();
                            if payload[0] == 0x30 && payload.len() == 8 {
                                // Possible recv ID? - It might pick up multiple IDs during the scan, we filter it later on
                                if let Some(r) = self.stage2_results.get_mut(&self.curr_scan_id) {
                                    r.push(frame.id)
                                } else {
                                    self.stage2_results
                                        .insert(self.curr_scan_id, vec![frame.id]);
                                }
                            }
                        }
                    }
                    Some(DiagScannerMessage::ScanPoll)
                }
            }
            3 => {
                if self.clock.elapsed().as_millis() > 100 {
                    println!("REM FILTER");
                    self.server.rem_can_filter(self.filter_idx);
                    self.server.clear_can_rx_buffer();
                    if self.curr_scan_id as usize >= self.stage2_results.len() {
                        self.server.close_can_interface(); // We don't need CAN anymore
                        return Some(DiagScannerMessage::IncrementStage); // Done with stage3 scan
                    }
                    let keys: Vec<u32> = self.stage2_results.keys().copied().collect();
                    let filter_id = self
                        .stage2_results
                        .get(&keys[self.curr_scan_id as usize])
                        .unwrap();
                    self.filter_idx = self
                        .server
                        .add_can_filter(commapi::comm_api::FilterType::Pass, filter_id[0], 0xFFFF)
                        .unwrap();
                    self.server.send_can_packets(
                        &[CanFrame::new(
                            keys[self.curr_scan_id as usize],
                            &[0x10, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
                        )],
                        0,
                    );
                    self.clock = Instant::now();
                    self.curr_scan_id += 1; // For next time
                    Some(DiagScannerMessage::ScanPoll)
                } else {
                    let keys: Vec<u32> = self.stage2_results.keys().copied().collect();
                    // Scanning current CAN ID entries
                    for frame in &self.server.read_can_packets(0, 10000).unwrap_or_default() {
                        let payload = frame.get_data();
                        if payload[0] == 0x30 && payload.len() == 8 {
                            // Not a false positive! We can add the Config to list!
                            self.stage3_results.push(ISO15765Config {
                                baud: 500000,
                                send_id: *keys.get((self.curr_scan_id - 1) as usize).unwrap(), // -1 is current scan ID whilst in this loop
                                recv_id: frame.id,
                                block_size: payload[1] as u32,
                                sep_time: payload[2] as u32,
                                use_ext_isotp: false,
                                use_ext_can: false
                            })
                        }
                    }
                    Some(DiagScannerMessage::ScanPoll)
                }
            }
            4 => {
                if self.clock.elapsed().as_millis() >= 5000 {
                    Some(DiagScannerMessage::IncrementStage)
                } else {
                    Some(DiagScannerMessage::ScanPoll)
                }
            }
            5 => {
                // KWP2000 scan
                if self.curr_scan_id as usize >= self.stage3_results.len() {
                    return Some(DiagScannerMessage::IncrementStage);
                }
                let ecu = self.stage3_results[self.curr_scan_id as usize];

                let mut ecu_res = ECUDiagSettings {
                    name: "Unknown ECU name".into(),
                    send_id: ecu.send_id,
                    flow_control_id: ecu.recv_id,
                    block_size: ecu.block_size,
                    sep_time_ms: ecu.sep_time,
                    uds_support: false,
                    kwp_support: false,
                };

                // Interrogate the ECU with extended diagnostic session
                match KWP2000ECU::start_diag_session(self.server.clone(), &ecu, None) {
                    Ok(mut s) => {
                        if let Ok(id) = read_dcx_mmc_id(&s) {
                            ecu_res.name = format!("ECU Part number: {}",id.part_number);
                            println!("ECU 0x{:04X} supports KWP2000!", ecu.send_id);
                            ecu_res.kwp_support = true;
                        }
                        s.exit_diag_session();
                    }
                    Err(e) => {
                        println!("KWP2000 server failed! {:?}", e);
                    }
                }
                self.curr_scan_id += 1;
                self.stage4_results.push(ecu_res);
                Some(DiagScannerMessage::ScanPoll)
            }
            6 => {
                // KWP2000 scan
                if self.curr_scan_id as usize >= self.stage3_results.len() {
                    return Some(DiagScannerMessage::IncrementStage);
                }
                let ecu = self.stage3_results[self.curr_scan_id as usize];
                // Interrogate the ECU with extended diagnostic session
                match UDSECU::start_diag_session(self.server.clone(), &ecu, None) {
                    Ok(mut s) => {
                        // TODO find a UDS only CMD to test with
                        println!("ECU 0x{:04X} supports UDS!", ecu.send_id);
                        self.stage4_results[self.curr_scan_id as usize].uds_support = true;
                        s.exit_diag_session();
                    }
                    Err(e) => {
                        println!("UDS server failed! {:?}", e);
                    }
                }
                self.curr_scan_id += 1;
                Some(DiagScannerMessage::ScanPoll)
            }
            _ => None,
        }
    }

    pub fn update(&mut self, msg: &DiagScannerMessage) -> Option<DiagScannerMessage> {
        self.status.clear();
        match msg {
            DiagScannerMessage::IncrementStage => self.increment_stage(),
            DiagScannerMessage::ScanPoll => self.poll(),
            DiagScannerMessage::SaveResults => {
                self.save_attempted = true;
                let v = VehicleECUList {
                    vehicle_name: "Unknown".into(),
                    vehicle_year: 0,
                    vehicle_brand: "Unknown".into(),
                    ecu_list: self.stage4_results.clone(),
                };
                let json = match serde_json::to_string_pretty(&v) {
                    Ok(j) => j,
                    Err(e) => {
                        self.status = format!("Error converting to JSON: {}", e);
                        return None;
                    }
                };
                let time = chrono::Utc::now();
                let path = std::env::current_dir()
                    .unwrap()
                    .join(format!("car_scan-{}.json", time.format("%F-%H_%M_%S")));
                let mut file = match File::create(&path) {
                    Ok(j) => j,
                    Err(e) => {
                        self.status = format!(
                            "Error creating save file {}: {}",
                            path.as_os_str().to_str().unwrap(),
                            e
                        );
                        return None;
                    }
                };
                match file.write_all(json.as_bytes()) {
                    Ok(_) => self.save_path = path.as_os_str().to_str().unwrap().into(),
                    Err(e) => {
                        self.status = format!(
                            "Error writing to save file {}: {}",
                            path.as_os_str().to_str().unwrap(),
                            e
                        );
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn view(&mut self) -> Element<DiagScannerMessage> {
        match self.curr_stage {
            0 => self.draw_stage_0(),
            1 => self.draw_stage_1(),
            2 => self.draw_stage_2(),
            3 => self.draw_stage_3(),
            4 => self.draw_stage_4(),
            5 => self.draw_stage_5(),
            6 => self.draw_stage_6(),
            7 => self.draw_stage_7(),
            _ => self.draw_stage_unk(),
        }
    }

    fn draw_stage_0(&mut self) -> Element<DiagScannerMessage> {
        let mut c = Column::new().padding(10).spacing(10).align_items(Align::Start).width(Length::Fill)
            .push(title_text("IMPORTANT", crate::themes::TitleSize::P2))
            .push(text("OpenVehicleDiag is going to scan your car for \
                KWP2000/UDS compatible ECUs that use ISO-TP. This will take some time. Before starting, please do the following:", TextType::Normal))
            .push(text("1. Ensure your battery is charged (Scan will terminate if battery voltage falls below 11.7V)", TextType::Normal))
            .push(text("2. Ensure your car is in the ignition position (Engine Off!)", TextType::Normal))
            .push(Space::with_height(Length::Units(50)))
            .push(text("If you see any warnings appear on your dashboard, do NOT panic!", TextType::Danger))
            .push(Space::with_height(Length::Units(50)))
            .push(button_coloured(&mut self.btn, "Start the scan", ButtonType::Warning).on_press(DiagScannerMessage::IncrementStage))
            .push(Space::with_height(Length::Units(50)));

        if !self.status.is_empty() {
            c = c
                .push(Row::new().width(Length::Fill).align_items(Align::Center))
                .push(text(
                    format!("Could not start scanning: {}", self.status).as_str(),
                    TextType::Danger,
                ))
        }
        c.into()
    }

    // Setting up the scanner
    fn draw_stage_1(&mut self) -> Element<DiagScannerMessage> {
        Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text(
                "Listening to existing CAN Traffic",
                crate::themes::TitleSize::P2,
            ))
            .push(progress_bar(
                0f32..=10000f32,
                self.clock.elapsed().as_millis() as f32,
                ButtonType::Info,
            ))
            .push(text(
                format!("Found {} existing can IDs", self.can_traffic_id_list.len()).as_str(),
                TextType::Normal,
            ))
            .into()
    }

    fn draw_stage_2(&mut self) -> Element<DiagScannerMessage> {
        let mut c = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text(
                "Scanning for ISO-TP ECUs (Stage 1/2)",
                crate::themes::TitleSize::P2,
            ))
            .push(progress_bar(
                0f32..=0x7FF as f32,
                self.curr_scan_id as f32,
                ButtonType::Info,
            ))
            .push(text(
                format!("Testing CAN ID 0x{:04X}", self.curr_scan_id).as_str(),
                TextType::Normal,
            ))
            .push(Space::with_height(Length::Units(20)));

        for (id, ls) in &self.stage2_results {
            c = c.push(text(
                format!(
                    "Found {} potential Receiver IDs for Sender 0x{:04X}",
                    ls.len(),
                    id
                )
                .as_str(),
                TextType::Normal,
            ))
        }
        if !self.status.is_empty() {
            c = c
                .push(Row::new().width(Length::Fill).align_items(Align::Center))
                .push(text(
                    format!("Error scanning: {}", self.status).as_str(),
                    TextType::Danger,
                ))
        }
        c.into()
    }

    fn draw_stage_3(&mut self) -> Element<DiagScannerMessage> {
        let mut c = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text(
                "Scanning for ISO-TP ECUs (Stage 2/2)",
                crate::themes::TitleSize::P2,
            ))
            .push(progress_bar(
                0f32..=self.stage2_results.len() as f32,
                self.curr_scan_id as f32,
                ButtonType::Info,
            ))
            .push(text("Interrogating...", TextType::Normal))
            .push(Space::with_height(Length::Units(20)));

        for cfg in &self.stage3_results {
            c = c.push(text(format!("ISO-TP Config generated!: Send ID: 0x{:04X}, Recv ID: 0x{:04X}, Block size: {}, Separation time: {}ms", cfg.send_id, cfg.recv_id, cfg.block_size, cfg.sep_time).as_str(), TextType::Normal))
        }
        if !self.status.is_empty() {
            c = c
                .push(Row::new().width(Length::Fill).align_items(Align::Center))
                .push(text(
                    format!("Error scanning: {}", self.status).as_str(),
                    TextType::Danger,
                ))
        }
        c.into()
    }

    fn draw_stage_4(&mut self) -> Element<DiagScannerMessage> {
        Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text(
                "Network cool down",
                crate::themes::TitleSize::P2,
            ))
            .push(progress_bar(
                0f32..=5000f32,
                self.clock.elapsed().as_millis() as f32,
                ButtonType::Info,
            ))
            .into()
    }

    fn draw_stage_5(&mut self) -> Element<DiagScannerMessage> {
        Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text(
                "Testing ECUs for UDS Capabilities",
                crate::themes::TitleSize::P2,
            ))
            .push(progress_bar(
                0f32..=self.stage3_results.len() as f32,
                self.curr_scan_id as f32,
                ButtonType::Info,
            ))
            .push(text(
                format!(
                    "Testing {}/{}",
                    self.curr_scan_id,
                    self.can_traffic_id_list.len()
                )
                .as_str(),
                TextType::Normal,
            ))
            .into()
    }

    fn draw_stage_6(&mut self) -> Element<DiagScannerMessage> {
        Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text(
                "Testing ECUs for UDS Capabilities",
                crate::themes::TitleSize::P2,
            ))
            .push(progress_bar(
                0f32..=self.stage3_results.len() as f32,
                self.curr_scan_id as f32,
                ButtonType::Info,
            ))
            .push(text(
                format!(
                    "Testing {}/{}",
                    self.curr_scan_id,
                    self.can_traffic_id_list.len()
                )
                .as_str(),
                TextType::Normal,
            ))
            .into()
    }

    fn draw_stage_7(&mut self) -> Element<DiagScannerMessage> {
        let mut c = Column::new()
            .padding(10)
            .spacing(10)
            .align_items(Align::Center)
            .width(Length::Fill)
            .push(title_text("Scan completed", crate::themes::TitleSize::P2));

        let mut row = Row::new();
        let mut list = Column::new().width(Length::Units(600));
        list = if self.stage4_results.is_empty() {
            list.push(text(
                "Unfortunately, no ISO-TP capable ECUs were found in your vehicle",
                TextType::Normal,
            ))
        } else {
            list.push(text(
                format!("Located {} ECUs", self.stage4_results.len()).as_str(),
                TextType::Normal,
            ))
        };
        for ecu in &self.stage4_results {
            list = list.push(text(
                format!(
                    "ECU 0x{:04X} - KWP2000?: {}, UDS?: {}",
                    ecu.send_id, ecu.kwp_support, ecu.uds_support
                )
                .as_str(),
                TextType::Normal,
            ));
        }
        row = row.push(list);

        // Allow the user to save the results to file
        let mut row2 = Column::new().width(Length::Fill);
        if !self.stage4_results.is_empty() {
            row2 = row2.push(text("Save results to file:", TextType::Normal));
            row2 = row2.push(
                button_outlined(&mut self.btn, "Save results to file", ButtonType::Info)
                    .on_press(DiagScannerMessage::SaveResults),
            );
            if self.save_attempted && !self.save_path.is_empty() {
                row2 = row2.push(text(
                    format!("Success, results saved to {}", self.save_path).as_str(),
                    TextType::Success,
                ));
                row2 = row2.push(text("You may now return home", TextType::Normal))
            } else if self.save_attempted && self.save_path.is_empty() && !self.status.is_empty() {
                // Error saving
                row2 = row2.push(text(
                    format!("Error saving results: {}", self.status).as_str(),
                    TextType::Warning,
                ));
            }
        }
        row = row.push(row2);
        c.push(row).into()
    }

    fn draw_stage_unk(&mut self) -> Element<DiagScannerMessage> {
        Row::new()
            .push(text(
                format!(
                    "Stage {} is unimplemented. Please report this!",
                    self.curr_stage
                )
                .as_str(),
                TextType::Normal,
            ))
            .into()
    }
}
