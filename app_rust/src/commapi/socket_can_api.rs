use crate::commapi;
use commapi::comm_api::ComServer;
use crate::commapi::comm_api::{ISO15765Data, CanFrame, FilterType, ComServerError, DeviceCapabilities};

use super::comm_api::Capability;

#[derive(Debug, Copy, Clone)]
pub struct SocketCanAPI {
    // TODO SocketCAN
}

#[allow(unused_variables)]
impl ComServer for SocketCanAPI {
    fn open_device(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn close_device(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn send_can_packets(&self, data: &[CanFrame], timeout_ms: u32) -> Result<usize, ComServerError> {
        unimplemented!()
    }

    fn read_can_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<CanFrame>, ComServerError> {
        unimplemented!()
    }

    fn send_iso15765_data(&self, data: &[ISO15765Data], timeout_ms: u32) -> Result<usize, ComServerError> {
        unimplemented!()
    }

    fn read_iso15765_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<ISO15765Data>, ComServerError> {
        unimplemented!()
    }

    fn open_can_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn close_can_interface(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn open_iso15765_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
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
        // Socket CAN cannot measure battery voltage, so return 12.0 just to keep OVD happy
        Ok(12.0)
    }

    fn clone_box(&self) -> Box<dyn ComServer> {
        unimplemented!()
    }

    fn get_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            name: "Socket CAN".into(),
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
        }
    }

    fn get_api(&self) -> &str {
        "Socket CAN"
    }

    fn is_connected(&self) -> bool {
        false
    }
}