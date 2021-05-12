use crate::commapi;
use crate::commapi::comm_api::{
    CanFrame, ComServerError, DeviceCapabilities, FilterType, ISO15765Data,
};
use commapi::comm_api::ComServer;

#[derive(Debug, Copy, Clone)]
pub struct DpduAPI {
    // TODO DPDU
}

#[allow(unused_variables)]
impl ComServer for DpduAPI {
    fn open_device(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn close_device(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn send_can_packets(
        &mut self,
        data: &[CanFrame],
        timeout_ms: u32,
    ) -> Result<usize, ComServerError> {
        unimplemented!()
    }

    fn read_can_packets(
        &self,
        timeout_ms: u32,
        max_msgs: usize,
    ) -> Result<Vec<CanFrame>, ComServerError> {
        unimplemented!()
    }

    fn send_iso15765_data(
        &self,
        data: &[ISO15765Data],
        timeout_ms: u32,
    ) -> Result<usize, ComServerError> {
        unimplemented!()
    }

    fn read_iso15765_packets(
        &self,
        timeout_ms: u32,
        max_msgs: usize,
    ) -> Result<Vec<ISO15765Data>, ComServerError> {
        unimplemented!()
    }

    fn open_can_interface(
        &mut self,
        bus_speed: u32,
        is_ext_can: bool,
    ) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn close_can_interface(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn open_iso15765_interface(
        &mut self,
        bus_speed: u32,
        is_ext_can: bool,
        ext_addressing: bool,
    ) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn add_can_filter(
        &mut self,
        f: FilterType) -> Result<u32, ComServerError> {
        unimplemented!()
    }

    fn rem_can_filter(&mut self, filter_idx: u32) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn add_iso15765_filter(&mut self, f: FilterType) -> Result<u32, ComServerError> {
        unimplemented!()
    }

    fn rem_iso15765_filter(&mut self, filter_idx: u32) -> Result<(), ComServerError> {
        unimplemented!()
    }

    fn set_iso15765_params(
        &mut self,
        separation_time_min: u32,
        block_size: u32,
    ) -> Result<(), ComServerError> {
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
        unimplemented!()
    }

    fn clone_box(&self) -> Box<dyn ComServer> {
        unimplemented!()
    }

    fn get_capabilities(&self) -> DeviceCapabilities {
        unimplemented!()
    }

    fn get_api(&self) -> &str {
        unimplemented!()
    }

    fn is_connected(&self) -> bool {
        false
    }
}
