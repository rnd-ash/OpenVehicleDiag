use std::sync::{Arc, RwLock};

use crate::commapi;
use commapi::comm_api::ComServer;
use socketcan::CANSocket;
use crate::commapi::comm_api::{ISO15765Data, CanFrame, FilterType, ComServerError, DeviceCapabilities};

use super::comm_api::Capability;

#[derive(Debug, Copy, Clone)]
pub enum SocketCanIfaceError {

}

#[derive(Debug, Clone)]
pub struct SocketCanAPI {
    iface: String,
    sockcan_iface: Arc<RwLock<Option<socketcan::CANSocket>>>,
    // TODO SocketCAN
}

impl SocketCanAPI {
    pub fn new(iface: String) -> Self {
        Self {
            iface,
            sockcan_iface: Arc::new(RwLock::new(None))
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

    fn send_can_packets(&self, data: &[CanFrame], timeout_ms: u32) -> Result<usize, ComServerError> {
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

    fn read_can_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<CanFrame>, ComServerError> {
        unimplemented!()
    }

    fn send_iso15765_data(&self, data: &[ISO15765Data], _timeout_ms: u32) -> Result<usize, ComServerError> {
        unimplemented!()
    }

    fn read_iso15765_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<ISO15765Data>, ComServerError> {
        unimplemented!()
    }

    fn open_can_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError> {
        if self.sockcan_iface.read().unwrap().is_some() {
            self.close_can_interface()?;
        }
        // Open our socket CAN interface!
        let tp_socket = CANSocket::open(&self.iface).map_err(|x| ComServerError {
            err_code: 1,
            err_desc: x.to_string(),
        })?;
        tp_socket.set_nonblocking(true).map_err(|x| ComServerError {
            err_code: 1,
            err_desc: x.to_string(),
        })?; // Disable blocking
        *self.sockcan_iface.write().unwrap() = Some(tp_socket);
        Ok(())
    }

    fn close_can_interface(&mut self) -> Result<(), ComServerError> {
        if self.sockcan_iface.read().unwrap().is_none() {
            return Ok(()) // No socket to close
        }
        drop(self.sockcan_iface.write().unwrap()); // Dropping the socketCAN Iface closes it
        Ok(())
    }

    fn open_iso15765_interface(&mut self, bus_speed: u32, is_ext_can: bool, ext_addressing: bool) -> Result<(), ComServerError> {
        Ok(())
    }

    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError> {
       Ok(())
    }

    fn add_can_filter(&self, filter: FilterType, id: u32, mask: u32) -> Result<u32, ComServerError> {
        unimplemented!()
    }

    fn rem_can_filter(&self, filter_idx: u32) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn add_iso15765_filter(&self, id: u32, mask: u32, resp_id: u32) -> Result<u32, ComServerError> {
        unimplemented!()
    }

    fn rem_iso15765_filter(&self, filter_idx: u32) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn set_iso15765_params(&self, separation_time_min: u32, block_size: u32) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn clear_can_rx_buffer(&self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn clear_can_tx_buffer(&self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn clear_iso15765_rx_buffer(&self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn clear_iso15765_tx_buffer(&self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn read_battery_voltage(&self) -> Result<f32, ComServerError> {
        // Socket CAN cannot measure battery voltage, so return -1.0 so user knows its not supported
        // rather than spitting out an error.
        Ok(-1.0)
    }

    fn clone_box(&self) -> Box<dyn ComServer> {
        Box::new(
            self.clone()
        )
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
            battery_voltage: Capability::NA
        }
    }

    fn get_api(&self) -> &str {
        "Socket CAN"
    }

    fn is_connected(&self) -> bool {
        false
    }
}