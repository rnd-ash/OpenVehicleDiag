use std::{
    borrow::Borrow,
    sync::{Arc, PoisonError, RwLock},
    time::Instant,
};

use crate::commapi::comm_api::{
    CanFrame, ComServerError, DeviceCapabilities, FilterType, ISO15765Data,
};
use crate::{commapi, main};
use commapi::comm_api::ComServer;
use socketcan::{CANError, CANFilter, CANSocket, ConstructionError};
use socketcan_isotp::{IsoTpOptions, IsoTpSocket};

use super::comm_api::Capability;

#[derive(Debug, Copy, Clone)]
pub enum SocketCanIfaceError {}

#[derive(Clone)]
pub struct SocketCanAPI {
    iface: String,
    sockcan_iface: Arc<RwLock<Option<socketcan::CANSocket>>>,
    isotp_iface: Arc<RwLock<Option<socketcan_isotp::IsoTpSocket>>>,
    can_filters: [Option<CANFilter>; 10],
    isotp_in_use: bool,
    req_iso_tp_settings: (u32, bool, bool), // Baud, ext CAN, ext Addressing
                                            // TODO SocketCAN
}

impl std::fmt::Debug for SocketCanAPI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SocketCanAPI")
            .field("iface", &self.iface)
            .field("socketcan_iface", &self.sockcan_iface)
            .finish()
    }
}

impl From<ConstructionError> for ComServerError {
    fn from(x: ConstructionError) -> Self {
        Self {
            err_code: 0xFF,
            err_desc: x.to_string(),
        }
    }
}

impl From<std::io::Error> for ComServerError {
    fn from(x: std::io::Error) -> Self {
        Self {
            err_code: x.raw_os_error().unwrap_or_default() as u32,
            err_desc: x.to_string(),
        }
    }
}

impl From<socketcan_isotp::Error> for ComServerError {
    fn from(x: socketcan_isotp::Error) -> Self {
        match x {
            socketcan_isotp::Error::LookupError { source } => ComServerError {
                err_code: 2,
                err_desc: source.to_string(),
            },
            socketcan_isotp::Error::IOError { source } => ComServerError {
                err_code: 3,
                err_desc: source.to_string(),
            },
        }
    }
}

impl SocketCanAPI {
    pub fn new(iface: String) -> Self {
        Self {
            iface,
            sockcan_iface: Arc::new(RwLock::new(None)),
            isotp_iface: Arc::new(RwLock::new(None)),
            can_filters: [None; 10],
            isotp_in_use: false,
            req_iso_tp_settings: (0, false, false),
        }
    }
}

impl SocketCanAPI {
    fn write_filters(&mut self) -> Result<(), ComServerError> {
        let filters: Vec<CANFilter> = self
            .can_filters
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();

        self.run_can_iface(|s| s.set_filter(&filters).map_err(|x| x.into()))?;

        Ok(())
    }

    fn run_can_iface<T, F: Fn(&CANSocket) -> Result<T, ComServerError>>(
        &self,
        func: F,
    ) -> Result<T, ComServerError> {
        match self.sockcan_iface.read() {
            Ok(r) => match r.as_ref() {
                Some(x) => func(x),
                None => Err(ComServerError {
                    err_code: 99,
                    err_desc: "Can Socket was null!".into(),
                }),
            },
            Err(_) => Err(ComServerError {
                err_code: 99,
                err_desc: "Read guard failed on CAN Socket".into(),
            }),
        }
    }
}

#[allow(unused_variables)]
impl ComServer for SocketCanAPI {
    fn open_device(&mut self) -> Result<(), ComServerError> {
        Ok(()) // Device isn't opened in SocketCAN, just the interfaces
    }

    fn close_device(&mut self) -> Result<(), ComServerError> {
        Ok(())
    }

    fn send_can_packets(
        &mut self,
        data: &[CanFrame],
        timeout_ms: u32,
    ) -> Result<usize, ComServerError> {
        if let Some(socket) = self.sockcan_iface.read().unwrap().as_ref() {
            if timeout_ms == 0 {
                for x in data {
                    //socket.write_frame(x.into()).map_err(|x| ComServerError {
                    //    err_code: 1,
                    //    err_desc: x.to_string(),
                    //})?;
                }
            } else {
                //socket.write_frame_insist(x).map_err(|x| ComServerError {
                //    err_code: 1,
                //    err_desc: x.to_string(),
                //})?;
            }
            Ok(data.len())
        } else {
            Err(ComServerError {
                err_code: 2,
                err_desc: "SocketCAN interface not open".into(),
            })
        }
    }

    fn read_can_packets(
        &self,
        timeout_ms: u32,
        max_msgs: usize,
    ) -> Result<Vec<CanFrame>, ComServerError> {
        // Timeout is handled manually here!
        let mut res: Vec<CanFrame> = Vec::with_capacity(max_msgs);

        if timeout_ms == 0 {
            let v_timeout = 10;
            match &self.run_can_iface(|x| x.read_frame().map_err(|x| x.into())) {
                Ok(cf) => res.push(CanFrame::from(*cf)),
                Err(e) => {
                    return Ok(res); // Return what we have
                }
            }
            if res.len() == max_msgs {
                return Ok(res);
            }
        } else {
            let start = Instant::now();
            while start.elapsed().as_millis() <= timeout_ms as u128 {
                match &self.run_can_iface(|x| x.read_frame().map_err(|x| x.into())) {
                    Ok(cf) => res.push(CanFrame::from(*cf)),
                    Err(_) => {} // Ignore error when using timeout
                }
                if res.len() == max_msgs {
                    return Ok(res);
                }
            }
        }
        Ok(res)
    }

    fn send_iso15765_data(
        &self,
        data: &[ISO15765Data],
        _timeout_ms: u32,
    ) -> Result<usize, ComServerError> {
        match self.isotp_iface.write().unwrap().as_ref() {
            Some(i) => {
                for x in data {
                    i.write(&x.data)?;
                }
                Ok(data.len())
            }
            None => Err(ComServerError {
                err_code: 4,
                err_desc: "Socket CAN Interface null!".into(),
            }),
        }
    }

    fn read_iso15765_packets(
        &self,
        timeout_ms: u32,
        max_msgs: usize,
    ) -> Result<Vec<ISO15765Data>, ComServerError> {
        match self.isotp_iface.write().unwrap().as_mut() {
            Some(i) => {
                let mut res = Vec::new();
                if timeout_ms == 0 {
                    // Keep reading until error
                    while let Ok(buf) = i.read() {
                        res.push(ISO15765Data {
                            id: 0x0000, // TODO save this ID!
                            data: Vec::from(buf),
                            pad_frame: false,
                            ext_addressing: false,
                        });
                        if res.len() == max_msgs {
                            return Ok(res);
                        }
                    }
                    Ok(res)
                } else {
                    let start = Instant::now();
                    while start.elapsed().as_millis() <= timeout_ms as u128 {
                        if let Ok(buf) = i.read() {
                            res.push(ISO15765Data {
                                id: 0x0000, // TODO save this ID!
                                data: Vec::from(buf),
                                pad_frame: false,
                                ext_addressing: false,
                            })
                        }
                    }
                    Ok(res)
                }
            }
            None => Err(ComServerError {
                err_code: 4,
                err_desc: "Socket CAN Interface null!".into(),
            }),
        }
    }

    fn open_can_interface(
        &mut self,
        bus_speed: u32,
        is_ext_can: bool,
    ) -> Result<(), ComServerError> {
        if self.sockcan_iface.read().unwrap().is_some() {
            self.close_can_interface()?;
        }
        // Open our socket CAN interface!
        let tp_socket = CANSocket::open(&self.iface).map_err(|x| ComServerError {
            err_code: 1,
            err_desc: x.to_string(),
        })?;
        tp_socket
            .set_nonblocking(true)
            .map_err(|x| ComServerError {
                err_code: 1,
                err_desc: x.to_string(),
            })?; // Disable blocking
        *self.sockcan_iface.write().unwrap() = Some(tp_socket);
        Ok(())
    }

    fn close_can_interface(&mut self) -> Result<(), ComServerError> {
        if self.sockcan_iface.read().unwrap().is_none() {
            return Ok(()); // No socket to close
        }
        self.can_filters = [None; 10]; // Remove all filters
        drop(self.sockcan_iface.write().unwrap()); // Dropping the socketCAN Iface closes it
        Ok(())
    }

    fn open_iso15765_interface(
        &mut self,
        bus_speed: u32,
        is_ext_can: bool,
        ext_addressing: bool,
    ) -> Result<(), ComServerError> {
        if self.sockcan_iface.read().unwrap().is_some() {
            self.close_can_interface()?; // Must do this first!
        }
        self.isotp_in_use = true;
        self.req_iso_tp_settings = (bus_speed, is_ext_can, ext_addressing);
        Ok(())
    }

    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError> {
        self.isotp_in_use = false;
        self.req_iso_tp_settings = (0, false, false);
        self.isotp_iface.write().unwrap().take(); // Bye bye iso-tp
        Ok(())
    }

    fn add_can_filter(&mut self, f: FilterType) -> Result<u32, ComServerError> {
        // SocketCAN
        let mut apply_mask = 0;
        let mut apply_id = 0;
        match f {
            FilterType::Block { id, mask } => {}
            FilterType::Pass { id, mask } => {
                apply_id = id;
                apply_mask = mask;
            }
            FilterType::IsoTP { id, mask, fc } => {
                return Err(ComServerError {
                    err_code: 99,
                    err_desc: "Cannot apply a FlowControl filter to CAN".into(),
                })
            }
        }

        let f = CANFilter::new(apply_id, apply_mask)?;
        // Find a free ID
        let mut pos = 99;
        for x in 0..10usize {
            if self.can_filters[x].is_none() {
                pos = x;
                break;
            }
        }
        if pos == 99 {
            // No free filters
            return Err(ComServerError {
                err_code: 98,
                err_desc: "No free CAN Filters were found".into(),
            });
        }

        self.can_filters[pos] = Some(f);
        // Now write the filters
        if let Err(e) = self.write_filters() {
            self.can_filters[pos] = None; // Unset if filter set failed!
            return Err(ComServerError::from(e));
        }
        // Set was OK! return result
        Ok(pos as u32)
    }

    fn rem_can_filter(&mut self, filter_idx: u32) -> Result<(), ComServerError> {
        self.can_filters[filter_idx as usize] = None;
        self.write_filters()
    }

    fn add_iso15765_filter(&mut self, f: FilterType) -> Result<u32, ComServerError> {
        if self.isotp_iface.read().unwrap().is_some() {
            // Socket CAN only allows for 1 ISO-TP filter!
            return Err(ComServerError {
                err_code: 1,
                err_desc: "Socket CAN only allows for 1 ISO-TP filter!".into(),
            });
        }

        if let FilterType::IsoTP { id, mask, fc } = f {
            // Now try to setup the ISO-TP interface
            let iface = IsoTpSocket::open_with_opts(&self.iface, fc, id & mask, None, None, None)?;
            iface.set_nonblocking(true)?; // Request non blocking!
            *self.isotp_iface.write().unwrap() = Some(iface);
            Ok(1)
        } else {
            Err(ComServerError {
                err_code: 99,
                err_desc: "Cannot apply a pass/block filter to ISOTP".into(),
            })
        }
    }

    fn rem_iso15765_filter(&mut self, filter_idx: u32) -> Result<(), ComServerError> {
        self.isotp_iface.write().unwrap().take();
        Ok(())
    }

    fn set_iso15765_params(
        &mut self,
        separation_time_min: u32,
        block_size: u32,
    ) -> Result<(), ComServerError> {
        //unimplemented!()
        Ok(()) // SocketCAN will not do this - It can auto negotiate with the ECU
    }

    fn clear_can_rx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn clear_can_tx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn clear_iso15765_rx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn clear_iso15765_tx_buffer(&self) -> Result<(), ComServerError> {
        Ok(()) // Socket CAN does not do this
    }

    fn read_battery_voltage(&self) -> Result<f32, ComServerError> {
        // Socket CAN cannot measure battery voltage, so return -1.0 so user knows its not supported
        // rather than spitting out an error.
        Ok(-1.0)
    }

    fn clone_box(&self) -> Box<dyn ComServer> {
        Box::new(self.clone())
    }

    fn get_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            name: self.iface.clone(),
            vendor: "Unknown".into(),
            library_path: "N/A".into(),
            device_fw_version: "N/A".into(),
            library_version: "N/A".into(),
            j1850vpw: Capability::NA,
            j1850pwm: Capability::NA,
            can: Capability::Yes,
            iso15765: Capability::Yes,
            iso9141: Capability::NA,
            iso14230: Capability::NA,
            ip: Capability::NA,
            battery_voltage: Capability::NA,
        }
    }

    fn get_api(&self) -> &str {
        "Socket CAN"
    }

    fn is_connected(&self) -> bool {
        if self.sockcan_iface.read().unwrap().is_some() || self.isotp_in_use {
            true
        } else {
            false
        }
    }
}
