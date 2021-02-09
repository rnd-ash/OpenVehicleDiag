use crate::passthru::{PassthruDevice, PassthruDrv, DrvVersion};
use crate::commapi::comm_api::{ComServer, ISO15765Data, FilterType, CanFrame, ComServerError, DeviceCapabilities, Capability};
use J2534Common::{PassthruError, PASSTHRU_MSG, Protocol, IoctlID, SConfig, IoctlParam, SConfigList, ConnectFlags, TxFlag, Loggable};
use J2534Common::IoctlID::READ_VBATT;
use std::{os::raw::c_void, time::Instant};
use J2534Common::PassthruError::{ERR_INVALID_CHANNEL_ID, ERR_FAILED};
use J2534Common::FilterType::{PASS_FILTER, BLOCK_FILTER, FLOW_CONTROL_FILTER};
use std::sync::{Arc, Mutex, RwLock};


#[derive(Debug, Clone)]
pub struct PassthruApi {
    device: Arc<PassthruDevice>,
    driver: Arc<Mutex<PassthruDrv>>,
    caps: Arc<RwLock<Option<DeviceCapabilities>>>,
    device_idx: Arc<RwLock<u32>>,
    can_channel_idx: Arc<RwLock<Option<u32>>>,
    iso15765_channel_idx: Arc<RwLock<Option<u32>>>,
    iso9141_channel_idx: Arc<RwLock<Option<u32>>>,
}

impl ComServer for PassthruApi {

    fn open_device(&mut self) -> Result<(), ComServerError> {
        match self.driver.lock().unwrap().open() {
            Err(e) => Err(self.convert_error(e)),
            Ok(dev_id) => {
                let mut idx = self.device_idx.write().unwrap();
                *idx = dev_id;
                Ok(())
            }
        }
    }

    fn close_device(&mut self) -> Result<(), ComServerError> {
        self.driver.lock().unwrap().close(*self.device_idx.read().unwrap()).map_err(|e| self.convert_error(e))
    }

    fn send_can_packets(&self, data: &[CanFrame], timeout_ms: u32) -> Result<usize, ComServerError> {
        let channel_id = match *self.can_channel_idx.read().unwrap() {
            Some(id) => id,
            None => return Err(self.convert_error(ERR_INVALID_CHANNEL_ID))
        };
        let mut msgs : Vec<PASSTHRU_MSG> = data.iter().map(|cf| PassthruApi::can_frame_to_pt_msg(cf)).collect();
        self.driver.lock().unwrap().write_messages(channel_id, &mut msgs, timeout_ms).map_err(|e| self.convert_error(e))
    }

    fn read_can_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<CanFrame>, ComServerError> {
        let channel_id = match *self.can_channel_idx.read().unwrap() {
            Some(id) => id,
            None => return Err(self.convert_error(ERR_INVALID_CHANNEL_ID))
        };
        self.driver.lock().unwrap().read_messages(channel_id, max_msgs as u32, timeout_ms)
            .map(|read| {
                read.iter().map(|msg| { PassthruApi::pt_msg_to_can_frame(msg) }).filter_map(Option::Some).map(|x| {x.unwrap()}).collect()
            }).map_err(|e| self.convert_error(e))
    }

    fn send_iso15765_data(&self, data: &[ISO15765Data], timeout_ms: u32) -> Result<usize, ComServerError> {
        let channel_id = match *self.iso15765_channel_idx.read().unwrap() {
            Some(id) => id,
            None => return Err(self.convert_error(ERR_INVALID_CHANNEL_ID))
        };
        let mut msgs : Vec<PASSTHRU_MSG> = data.iter().map(|d| PassthruApi::iso15765_to_pt_msg(d)).collect();
        self.driver.lock().unwrap().write_messages(channel_id, &mut msgs, timeout_ms).map_err(|e| self.convert_error(e))
    }

    fn read_iso15765_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<ISO15765Data>, ComServerError> {
        let channel_id = match *self.iso15765_channel_idx.read().unwrap() {
            Some(id) => id,
            None => return Err(self.convert_error(ERR_INVALID_CHANNEL_ID))
        };

        
        let elapsed = Instant::now();
        let mut res: Vec<ISO15765Data> = Vec::new();
        while elapsed.elapsed().as_millis() <= timeout_ms as u128 {
            let t: Result<Vec<ISO15765Data>, PassthruError> = self.driver.lock().unwrap().read_messages(channel_id, 1, timeout_ms)
            .map(|read| {
                read
                .iter()
                .map(|msg| { PassthruApi::pt_msg_to_iso15765(msg) }).filter_map(Option::Some).map(|x| {x.unwrap()})
                .filter(|x| !x.data.is_empty())
                .collect()
            });
            match t {
                Ok(vec) => {
                    res.extend(vec);
                    if res.len() == max_msgs {
                        return Ok(res) // Max reached, return now!
                    }
                },
                Err(e) => {
                    if e != PassthruError::ERR_BUFFER_EMPTY {
                        return Err(self.convert_error(e))
                    }
                }
            }
        }
        Ok(res) // Return what we have
    }

    fn open_can_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError> {
        if self.can_channel_idx.read().unwrap().is_some() {
            // Already open, close first, maybe going from non ext to ext can
            self.close_can_interface()?;
        }
        let mut flags: u32 = 0;
        if is_ext_can {
            flags |= ConnectFlags::CAN_29BIT_ID as u32;
        }
        let channel_id = self.driver.lock().unwrap().connect(*self.device_idx.read().unwrap(), Protocol::CAN, flags, bus_speed).map_err(|e| self.convert_error(e))?;
        *self.can_channel_idx.write().unwrap() = Some(channel_id);
        *self.iso15765_channel_idx.write().unwrap() = None; // Physically impossible to have both CAN and ISOTP enabled at the same time
        Ok(())
    }

    fn close_can_interface(&mut self) -> Result<(), ComServerError> {
        if let Ok(mut lock) = self.can_channel_idx.write() {
            if lock.is_none() {
                return Ok(())
            }
            self.driver.lock().unwrap().disconnect(lock.unwrap()).map_err(|e| self.convert_error(e))?;
            *lock = None;
        }
        Ok(())
    }

    fn open_iso15765_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError> {
        if self.iso15765_channel_idx.read().unwrap().is_some() {
            // Already open, close first, maybe going from non ext to ext can
            self.close_iso15765_interface()?;
        }
        let mut flags: u32 = 0;
        if is_ext_can {
            flags |= ConnectFlags::CAN_29BIT_ID as u32;
        }
        let channel_id = self.driver.lock().unwrap().connect(*self.device_idx.read().unwrap(), Protocol::ISO15765, flags, bus_speed).map_err(|e| self.convert_error(e))?;
        *self.iso15765_channel_idx.write().unwrap() = Some(channel_id);
        *self.can_channel_idx.write().unwrap() = None; // Physically impossible to have both CAN and ISOTP enabled at the same time
        Ok(())

    }

    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError> {
        if let Ok(mut lock) = self.iso15765_channel_idx.write() {
            if lock.is_none() {
                return Ok(())
            }
            self.driver.lock().unwrap().disconnect(lock.unwrap()).map_err(|e| self.convert_error(e))?;
            *lock = None;
        }
        Ok(())
    }

    fn add_can_filter(&self, filter: FilterType, id: u32, mask: u32) -> Result<u32, ComServerError> {
        match *self.can_channel_idx.read().unwrap() {
            None => Err(self.convert_error(ERR_INVALID_CHANNEL_ID)),
            Some(idx) => {
                let f_type = match filter {
                    FilterType::Pass => PASS_FILTER,
                    FilterType::Block => BLOCK_FILTER
                };

                let mut mask_msg = PASSTHRU_MSG {
                    protocol_id: Protocol::CAN as u32,
                    data_size: 4,
                    ..Default::default()
                };
                PassthruApi::u32_to_msg_id(mask, &mut mask_msg);

                let mut ptn_msg =  PASSTHRU_MSG {
                    protocol_id: Protocol::CAN as u32,
                    data_size: 4,
                    ..Default::default()
                };
                PassthruApi::u32_to_msg_id(id, &mut ptn_msg);
                self.driver.lock().unwrap().start_msg_filter(idx, f_type, &mask_msg, &ptn_msg, None).map_err(|e| self.convert_error(e))
            }
        }
    }

    fn rem_can_filter(&self, filter_idx: u32) -> Result<(), ComServerError> {
        match *self.iso15765_channel_idx.read().unwrap() {
            None => Ok(()), // OK as filter has already been deleted when channel was destroyed
            Some(id) => self.driver.lock().unwrap().stop_msg_filter(id, filter_idx).map_err(|e| self.convert_error(e))
        }
    }

    fn add_iso15765_filter(&self, id: u32, mask: u32, flow_control_id: u32) -> Result<u32, ComServerError> {
        match *self.iso15765_channel_idx.read().unwrap() {
            None => Err(self.convert_error(ERR_INVALID_CHANNEL_ID)),
            Some(idx) => {
                let mut mask_msg = PASSTHRU_MSG {
                    protocol_id: Protocol::ISO15765 as u32,
                    data_size: 4,
                    ..Default::default()
                };
                PassthruApi::u32_to_msg_id(mask, &mut mask_msg);

                let mut ptn_msg = PASSTHRU_MSG {
                    protocol_id: Protocol::ISO15765 as u32,
                    data_size: 4,
                    ..Default::default()
                };
                PassthruApi::u32_to_msg_id(id, &mut ptn_msg);

                let mut fc_msg = PASSTHRU_MSG {
                    protocol_id: Protocol::ISO15765 as u32,
                    data_size: 4,
                    ..Default::default()
                };
                PassthruApi::u32_to_msg_id(flow_control_id, &mut fc_msg);
                self.driver.lock().unwrap().start_msg_filter(idx, FLOW_CONTROL_FILTER, &mask_msg, &ptn_msg, Some(fc_msg)).map_err(|e| self.convert_error(e))
            }
        }
    }

    fn rem_iso15765_filter(&self, filter_idx: u32) -> Result<(), ComServerError> {
        match *self.iso15765_channel_idx.read().unwrap() {
            None => Ok(()), // Return OK if the channel no longer exists since the filter has already been removed
            Some(idx) => self.driver.lock().unwrap().stop_msg_filter(idx, filter_idx).map_err(|e| self.convert_error(e))
        }
    }

    fn set_iso15765_params(&self, separation_time_min: u32, block_size: u32) -> Result<(), ComServerError> {
        let channel_id = match *self.iso15765_channel_idx.read().unwrap() {
            Some(idx) => idx,
            None => return Err(self.convert_error(ERR_INVALID_CHANNEL_ID))
        };
        let mut params = [
            SConfig { parameter: IoctlParam::ISO15765_STMIN as u32, value: separation_time_min },
            SConfig { parameter: IoctlParam::ISO15765_BS as u32, value: block_size }
        ];

        let mut sconfig_list = SConfigList {
            num_of_params: 2,
            config_ptr: params.as_mut_ptr()
        };
        self.driver.lock().unwrap().ioctl(
            channel_id,
            IoctlID::SET_CONFIG,
            (&mut sconfig_list) as *mut _ as *mut c_void ,
            std::ptr::null_mut()
        ).map_err(|e| self.convert_error(e))
    }

    fn clear_can_rx_buffer(&self) -> Result<(), ComServerError> {
        match *self.can_channel_idx.read().unwrap() {
            Some(idx) => {
                self.driver.lock().unwrap().ioctl(idx, IoctlID::CLEAR_RX_BUFFER, std::ptr::null_mut(), std::ptr::null_mut())
            }
            None => Ok(())
        }.map_err(|e| self.convert_error(e))
    }

    fn clear_can_tx_buffer(&self) -> Result<(), ComServerError> {
        match *self.can_channel_idx.read().unwrap() {
            Some(idx) => {
                self.driver.lock().unwrap().ioctl(idx, IoctlID::CLEAR_TX_BUFFER, std::ptr::null_mut(), std::ptr::null_mut())
            }
            None => Ok(())
        }.map_err(|e| self.convert_error(e))
    }

    fn clear_iso15765_rx_buffer(&self) -> Result<(), ComServerError> {
        match *self.iso15765_channel_idx.read().unwrap() {
            Some(idx) => {
                self.driver.lock().unwrap().ioctl(idx, IoctlID::CLEAR_RX_BUFFER, std::ptr::null_mut(), std::ptr::null_mut())
            }
            None => Ok(())
        }.map_err(|e| self.convert_error(e))
    }

    fn clear_iso15765_tx_buffer(&self) -> Result<(), ComServerError> {
        match *self.iso15765_channel_idx.read().unwrap() {
            Some(idx) => {
                self.driver.lock().unwrap().ioctl(idx, IoctlID::CLEAR_TX_BUFFER, std::ptr::null_mut(), std::ptr::null_mut())
            }
            None => Ok(())
        }.map_err(|e| self.convert_error(e))
    }

    fn read_battery_voltage(&self) -> Result<f32, ComServerError> {
        let mut output = 0;
        self.driver.lock().unwrap().ioctl(
            *self.device_idx.read().unwrap(),
            READ_VBATT,
            std::ptr::null_mut(),
            (&mut output) as *mut _ as *mut c_void
        ).map(|_| { output as f32 / 1000.0 }).map_err(|e| self.convert_error(e))
    }

    fn clone_box(&self) -> Box<dyn ComServer> {
        Box::new(
            Self {
                device: self.device.clone(),
                driver: self.driver.clone(),
                caps: self.caps.clone(),
                device_idx: self.device_idx.clone(),
                can_channel_idx: self.can_channel_idx.clone(),
                iso15765_channel_idx: self.iso15765_channel_idx.clone(),
                iso9141_channel_idx: self.iso9141_channel_idx.clone()
            }
        )
    }

    fn get_capabilities(&self) -> DeviceCapabilities {
        if let Some(caps) = self.caps.read().unwrap().as_ref() {
            return caps.clone();
        }
        let version = self.driver.lock().unwrap().get_version(*self.device_idx.read().unwrap()).unwrap_or(
            DrvVersion {
                dll_version: "Unknown".into(),
                api_version: "Unknown".into(),
                fw_version: "Unknown".into()
            }
        );
        let caps = DeviceCapabilities {
            name: self.device.name.clone(),
            library_version: version.dll_version.clone(),
            device_fw_version: version.fw_version,
            vendor: self.device.vendor.clone(),
            library_path: self.device.drv_path.clone(),
            j1850vpw: Capability::from_bool(self.device.j1850vpw),
            j1850pwm: Capability::from_bool(self.device.j1850pwm),
            can: Capability::from_bool(self.device.can),
            iso15765: Capability::from_bool(self.device.iso15765),
            iso9141: Capability::from_bool(self.device.iso9141),
            iso14230: Capability::from_bool(self.device.iso14230),
            ip: Capability::NA
        };
        *self.caps.write().unwrap() = Some(caps.clone());
        caps
    }

    fn get_api(&self) -> &str {
        "SAE J2534"
    }

    fn is_connected(&self) -> bool {
        return self.iso15765_channel_idx.read().unwrap().is_some() ||
            self.can_channel_idx.read().unwrap().is_some() ||
            self.iso9141_channel_idx.read().unwrap().is_some()
    }
}

impl PassthruApi {
    pub fn new(desc: PassthruDevice, driver: PassthruDrv) -> Self {
        Self {
            device: Arc::from(desc),
            driver: Arc::from(Mutex::new(driver)),
            caps: Arc::new(Default::default()),
            device_idx: Arc::from(RwLock::new(0)),
            can_channel_idx: Arc::from(RwLock::new(None)),
            iso15765_channel_idx: Arc::from(RwLock::new(None)),
            iso9141_channel_idx: Arc::from(RwLock::new(None)),
        }
    }

    fn can_frame_to_pt_msg(cf: &CanFrame) -> PASSTHRU_MSG {
        let mut msg = PASSTHRU_MSG {
            protocol_id: Protocol::CAN as u32,
            data_size: cf.dlc as u32 + 4, // +4 for CAN ID
            ..Default::default()
        };
        PassthruApi::u32_to_msg_id(cf.id, &mut msg);
        msg.data[4..msg.data_size as usize].copy_from_slice(cf.get_data());
        msg
    }

    /// Converts a PASSTHRU_MSG to a Can Frame
    ///
    fn pt_msg_to_can_frame(msg: &PASSTHRU_MSG) -> Option<CanFrame> {
        if msg.protocol_id != Protocol::CAN as u32 || msg.data_size < 4 {
            return None;
        }
        let data = &msg.data[4..msg.data_size as usize];
        Some(CanFrame::new(PassthruApi::msg_id_to_u32(msg), data))
    }

    fn pt_msg_to_iso15765(msg: &PASSTHRU_MSG) -> Option<ISO15765Data> {
        if msg.protocol_id != Protocol::ISO15765 as u32 || msg.data_size < 4 {
            return None;
        }
        Some(
            ISO15765Data {
                id: PassthruApi::msg_id_to_u32(msg),
                data: Vec::from(&msg.data[4..msg.data_size as usize]),
                pad_frame: false
            }
        )
    }

    fn iso15765_to_pt_msg(d: &ISO15765Data) -> PASSTHRU_MSG {
        let mut msg = PASSTHRU_MSG {
            protocol_id: Protocol::ISO15765 as u32,
            data_size: d.data.len() as u32 + 4, // +4 for CAN ID
            ..Default::default()
        };
        PassthruApi::u32_to_msg_id(d.id, &mut msg);
        msg.data[4..msg.data_size as usize].copy_from_slice(d.data.as_slice());
        if d.pad_frame {
            msg.tx_flags = TxFlag::ISO15765_FRAME_PAD.bits();
        }
        msg
    }

    #[inline(always)]
    fn msg_id_to_u32(m: &PASSTHRU_MSG) -> u32 {
        (m.data[0] as u32) << 24 | (m.data[1] as u32) << 16 | (m.data[2] as u32) << 8 | m.data[3] as u32
    }

    #[inline(always)]
    fn u32_to_msg_id(i: u32, msg: &mut PASSTHRU_MSG) {
        msg.data[0] = (i >> 24) as u8;
        msg.data[1] = (i >> 16) as u8;
        msg.data[2] = (i >>  8) as u8;
        msg.data[3] = i as u8;
    }

    fn convert_error(&self, e: PassthruError) -> ComServerError {
        let code = e as u32;
        let desc = if e == ERR_FAILED {
            if let Ok(desc) = self.driver.lock().unwrap().get_last_error() {
                desc
            } else {
                "Generic unknown failure".into()
            }
        } else {
            e.to_string().into()
        };
        ComServerError { err_code: code, err_desc: desc }
    }
}
