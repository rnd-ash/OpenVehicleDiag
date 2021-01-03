use J2534Common::{PASSTHRU_MSG, PassthruError, Protocol, RxFlag, TxFlag};
/// This file is only for random tests before i implement them in the JS framework
use passthru::*;
use std::{collections::{HashSet, VecDeque}, sync::Arc};
use crate::{Device, passthru};
use core::panic;

type Result<X> = std::result::Result<X, PassthruError>;

#[cfg(test)]
fn run_test_on_device<X, T: FnOnce(&PassthruDrv) -> Result<X>>(op: T) -> Result<X> {
    let drv = match passthru::PassthruDevice::find_all() {
        Ok(s) => s[0].clone(),
        Err(e) => {
            panic!("No passthru devices available!");
        }
    };
    if let Ok(device) = PassthruDrv::load_lib(drv.drv_path) {
        op(&device)
    } else {
        panic!("Library loading failed!");
    }
}

/*
#[test]
pub fn test_fuzzing() {
    run_test_on_device(|dev| {
        let dev_id = dev.open()?;
        let mut channel_id = dev.connect(dev_id, J2534Common::Protocol::CAN, 0, 500_000)?;

        // Begin fuzzing
        println!("Begin fuzzing!");
        let mut send_msg = PASSTHRU_MSG::default();
        send_msg.data_size = 12;
        send_msg.protocol_id = Protocol::CAN as u32;
        // Set an open CAN Filter so we can listen to all frames on the bus!
        
        // Mask message
        let mut mask_msg = PASSTHRU_MSG::default();
        mask_msg.data_size = 4;
        mask_msg.data[..4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        // Mask message
        let mut ptn_msg = PASSTHRU_MSG::default();
        ptn_msg.data_size = 4;
        ptn_msg.data[..4].copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);

        let mut filter_id = dev.start_msg_filter(channel_id, J2534Common::FilterType::PASS_FILTER, &mask_msg, &ptn_msg, None)?;

        let mut res: Vec<String> = Vec::new();
        let mut flip = false;

        // First get a list of all CIDs on the vehicle bus, and ignore them
        println!("Scanning for already present CIDs");
        let mut start = std::time::Instant::now();
        let mut ignore_ids: Vec<u32> = Vec::new();

        // Wake up the ODB-II port
        send_msg.data[..12].copy_from_slice(&[0x00, 0x00, 0x00, 0x1C, 0x02, 0x10, 0x92, 0x00, 0x00, 0x00, 0x00, 0x00]);
        dev.write_messages(channel_id, &mut [send_msg], 0);

        while start.elapsed().as_millis() <= 5000 {
            if let Ok(read) = dev.read_messages(channel_id, 10, 0) {
                if read.len() > 0 {
                    read.iter().for_each(|msg| {
                        if msg.data_size == 12 {
                            let id: u32 = (msg.data[0] as u32) << 24 | (msg.data[1] as u32) << 16 | (msg.data[2] as u32) << 8 | msg.data[3] as u32;
                            ignore_ids.push(id);
                        }
                    });
                }
            }
        }
        dev.disconnect(channel_id)?; // Close the channel to stop flooding messages
        // Make it unique
        ignore_ids = ignore_ids.iter().cloned().collect::<HashSet<u32>>().into_iter().collect();
        println!("Ignoring {} CIDs", ignore_ids.len());
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let mut test_ids: VecDeque<u32> = VecDeque::new();
        for tmp in 0..=0x7FF as u32 {
            if !ignore_ids.contains(&tmp) {
                test_ids.push_back(tmp);
            }
        }

        let total_to_test = test_ids.len();
        let mut ids_tested = 0;


        // Reopen the channel!
        let channel_id = dev.connect(dev_id, J2534Common::Protocol::CAN, 0, 500_000)?;
        let filter_id = dev.start_msg_filter(channel_id, J2534Common::FilterType::PASS_FILTER, &mask_msg, &ptn_msg, None)?;
        loop { // 11bit
            // Finished testing
            if test_ids.len() == 0 {
                break;
            }

            // Flip the CAN ID every time.
            // This ensures that if 2 ISO15765 ID's come from the same ECU, and thus are next to each other,
            // that the ECU will timeout before the next request ID Comes in, allowing it to respond.
            let cid = match flip {
                true => test_ids.pop_front().unwrap(),
                false => test_ids.pop_back().unwrap()
            };
            flip = !flip;

            println!("Scan progress: {}%. Curr test ID: 0x{:04X}", (ids_tested as f32 / total_to_test as f32) * 100.0, cid);
            send_msg.data[..12].copy_from_slice(&[(cid >> 24) as u8, (cid >> 16) as u8, (cid >> 8) as u8, (cid & 0xFF) as u8, 0x10, 0x16, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

            if let Err(e) = dev.write_messages(channel_id, &mut [send_msg], 0) {
                eprintln!("Error sending message. {}", e as u32);
                break;
            }
            // Wait for 50ms for response from ECU
            let start = std::time::Instant::now();
            while start.elapsed().as_millis() <= 50 {
                if let Ok(read) = dev.read_messages(channel_id, 10, 0) {
                    if read.len() > 0 {
                        read.iter().for_each(|msg| {
                            if msg.data_size == 12 {
                                let resp_can_id: u32 = (msg.data[0] as u32) << 24 | (msg.data[1] as u32) << 16 | (msg.data[2] as u32) << 8 | msg.data[3] as u32;
                                if msg.data[4] & 0xF0 == 0x30 { // Check flow control nibble
                                    if !ignore_ids.contains(&resp_can_id) { // Is this ID actually just part of normal car communication?
                                        println!("POSSIBLE RESPONSE FRAME: CID: 0x{:04X}, Data: {:02X?}", resp_can_id, &msg.data[4..12]);
                                        res.push(format!("REQ: 0x{:04X}. RESP: 0x{:04X} ({:02X?})", cid, resp_can_id, &msg.data[4..12]));
                                    }
                                }
                            }
                        });
                    }
                }
            }
            ids_tested += 1;
        };

        println!("RESULTS:");
        for s in res {
            println!("{}",s);
        }

        dev.stop_msg_filter(channel_id, filter_id)?;
        dev.disconnect(channel_id)?;
        dev.close(dev_id)?;
        Ok(())
    }).unwrap();
}
*/

#[test]
fn test_odb_can() {
    run_test_on_device(|dev| {
        let dev_id = dev.open()?;
        let channel_id = dev.connect(dev_id, J2534Common::Protocol::ISO15765, 0, 500_000)?;

        // Mask message
        let mut mask_msg = PASSTHRU_MSG::default();
        mask_msg.data_size = 4;
        mask_msg.data[..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);

        // Mask message
        let mut ptn_msg = PASSTHRU_MSG::default();
        ptn_msg.data_size = 4;
        ptn_msg.data[..4].copy_from_slice(&[0x00, 0x00, 0x07, 0xE8]);

        // FC message
        let mut flow_msg = PASSTHRU_MSG::default();
        flow_msg.data_size = 4;
        flow_msg.data[..4].copy_from_slice(&[0x00, 0x00, 0x07, 0xDF]);

        let filter_id = dev.start_msg_filter(channel_id, J2534Common::FilterType::FLOW_CONTROL_FILTER, &mask_msg, &ptn_msg, Some(flow_msg))?;


        let send_payload = [0x00, 0x00, 0x07, 0xDF, 0x09, 0x02];
        let mut send_msg = PASSTHRU_MSG::default();
        send_msg.data_size = send_payload.len() as u32;
        send_msg.protocol_id = Protocol::ISO15765 as u32;
        send_msg.tx_flags = TxFlag::ISO15765_FRAME_PAD.bits();
        send_msg.data[..send_payload.len()].copy_from_slice(&send_payload);
        dev.write_messages(channel_id, &mut[send_msg], 0);
        let start = std::time::Instant::now();
        while start.elapsed().as_millis() <= 50 {
            if let Ok(read) = dev.read_messages(channel_id, 1, 0) {
                if read.len() > 0 {
                    if read[0].rx_status & RxFlag::ISO15765_FIRST_FRAME.bits() > 0 {
                        println!("Received first frame. ECU is providing a response!");
                    } else {
                        let payload = &read[0].data[4..read[0].data_size as usize];
                        println!("RESPONSE: {:02X?} in {} ms", payload, start.elapsed().as_millis());
                        println!("VIN: {}", String::from_utf8(payload[1..].to_vec()).unwrap());
                        break;
                    }

                }
            }
        }

        dev.close(dev_id).is_ok();
        Ok(())
    }).unwrap();
}