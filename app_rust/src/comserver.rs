use std::{ffi::c_void, fmt};

use fmt::write;
use J2534Common::{ConnectFlags, PASSTHRU_MSG, PassthruError, Protocol, RxFlag, TxFlag};

use crate::passthru::{PassthruDevice, PassthruDrv};

#[derive(Debug, Copy, Clone)]
struct CanChannel {
    id: u32,
    baud: u32,
    is_extended: bool,
}
unsafe impl Send for CanChannel {}
unsafe impl Sync for CanChannel {}

#[derive(Debug, Copy, Clone)]
pub struct CanFrame {
    id: u32,
    dlc: u8,
    data: [u8; 8],
    is_extended: bool,
}

impl CanFrame {
    /// Creates a new Can Frame
    /// # Panics
    /// * panics if data size is greater than 8
    fn new(id: u32, data: &[u8], is_extended: bool) -> Self {
        if data.len() > 8 {
            panic!("CAN Frame data cannot be more than 8 bytes")
        }
        let mut can_data: [u8; 8] = [0; 8];
        can_data[0..data.len()].copy_from_slice(data);
        Self {
            id,
            dlc: data.len() as u8,
            data: can_data,
            is_extended,
        }
    }

    /// Creates a new CAN Frame from a raw byte array
    /// The first 4 bytes of this array are treated as the CAN ID,
    /// regardless as to weather the frame is extended or not
    /// # Panics
    /// * Panics if data size is greater than 12, or if data size is smaller than 4
    fn from_byte_array(data: &[u8], is_extended: bool) -> Self {
        if data.len() < 4 || data.len() > 12 {
            panic!("Invalid data length for CAN Frame")
        }
        let id: u32 = (data[0] as u32) << 24
            | (data[1] as u32) << 16
            | (data[2] as u32) << 8
            | (data[3] as u32);
        let dlc = data.len() - 4;
        let mut can_data: [u8; 8] = [0; 8];

        can_data[0..dlc].copy_from_slice(&data[4..4 + dlc]);
        Self {
            id,
            dlc: dlc as u8,
            data: can_data,
            is_extended,
        }
    }

    /// Converts the CAN Frame to a [PASSTHRU_MSG] which can be transmitted to a J2534 device
    fn to_pt_msg(&self) -> PASSTHRU_MSG {
        let mut msg = PASSTHRU_MSG::default();
        msg.data_size = 4 + self.dlc as u32;
        if self.is_extended {
            msg.tx_flags = TxFlag::CAN_EXTENDED_ID.bits();
        }
        // CID copy
        msg.data[0] = (self.id >> 24) as u8;
        msg.data[1] = (self.id >> 16) as u8;
        msg.data[2] = (self.id >> 8) as u8;
        msg.data[3] = (self.id >> 0) as u8;
        // Copy CAN Data
        msg.data[4..12].copy_from_slice(&self.data);
        return msg;
    }
}

impl fmt::Display for CanFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            format!(
                "ID: 0x{:04X} ({}) Data: {:02X?}",
                self.id,
                if self.is_extended { "STD" } else { "EXT" },
                &self.data[0..self.dlc as usize]
            )
            .as_str(),
        )
    }
}

unsafe impl Send for CanFrame {}
unsafe impl Sync for CanFrame {}

#[derive(Debug, Clone)]
pub struct ComServer {
    device_info: PassthruDevice,
    device_driver: PassthruDrv,
    device_id: u32,
    can_channel: Option<CanChannel>,
}
unsafe impl Send for ComServer {}
unsafe impl Sync for ComServer {}

#[derive(Debug, Clone)]
pub struct Error {
    err_type: PassthruError,
    err_desc: Option<String>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.err_desc {
            Some(desc) => write!(f, "{:?}({})", self.err_type, desc),
            None => write!(f, "{:?}", self.err_type),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl ComServer {
    pub fn new(info: PassthruDevice, driver: PassthruDrv, idx: u32) -> Self {
        Self {
            device_info: info,
            device_driver: driver,
            device_id: idx,
            can_channel: None,
        }
    }

    fn convert_error(&self, e: PassthruError) -> Error {
        if e == PassthruError::ERR_FAILED {
            // Try and get the description from the device as to what went wrong
            if let Ok(desc) = self.device_driver.get_last_error() {
                Error {
                    err_type: e,
                    err_desc: Some(desc),
                }
            } else {
                // Err failed was thrown, but driver does not have a description
                Error {
                    err_type: e,
                    err_desc: Some(String::from("Unknown driver error")),
                }
            }
        } else {
            Error {
                err_type: e,
                err_desc: None,
            }
        }
    }

    /// Returns the battery voltage of the vehicle
    pub fn get_batt_voltage(&self) -> Result<f32> {
        let mut output: u32 = 0;
        self.device_driver
            .ioctl(
                self.device_id,
                J2534Common::IoctlID::READ_VBATT,
                std::ptr::null_mut(),
                (&mut output) as *mut _ as *mut c_void,
            )
            .map_err(|e| self.convert_error(e))
            .map(|_| output as f32 / 1000.0)
    }

    // More dev oriented functions that allow for sending specific payloads to a vehicle
    // Using the J2534 API.
    // Alternatively, this can be modified to send data using future D-PDU

    /// Creates a new CAN Channel
    pub fn create_can_channel(&mut self, baudrate: u32, is_extended: bool) -> Result<()> {
        let mut flags = 0;
        if is_extended {
            flags = ConnectFlags::CAN_29BIT_ID as u32;
        }

        let channel =
            self.device_driver
                .connect(self.device_id, J2534Common::Protocol::CAN, flags, baudrate);
        match channel {
            Ok(c) => {
                self.can_channel = Some(CanChannel {
                    id: c,
                    baud: baudrate,
                    is_extended,
                });
                return Ok(());
            }
            Err(e) => return Err(self.convert_error(e)),
        }
    }

    pub fn destroy_can_channel(&mut self) -> Result<()> {
        if let Some(channel) = self.can_channel {
            let res = self.device_driver.disconnect(channel.id).map_err(|e| self.convert_error(e));
            if res.is_ok() {
                self.can_channel = None;
            }
            return res;
        }
        Ok(()) // No channel to destroy, its OK
    }



    /// Sends a list of CAN packets to the car, and does not ask the adapter for a conformation
    /// that all the packet were sent successfully
    ///
    /// # Params
    /// * frames - List of frames to be transmitted
    ///
    /// # Panics
    /// * Panics if the devices CAN channel is not yet configured
    pub fn send_can_packets(&self, frames: &[CanFrame]) -> Result<()> {
        let channel_id = match self.can_channel.as_ref() {
            Some(channel) => channel.id,
            None => panic!("CAN Channel has not been configured!"),
        };
        let mut msgs: Vec<PASSTHRU_MSG> = frames.iter().map(|f| f.to_pt_msg()).collect();
        self.device_driver
            .write_messages(channel_id, &mut msgs, 0)
            .map_err(|e| self.convert_error(e))
            .map(|_| ())
    }

    /// Sends a list of CAN Packets to the car and verifies that they were all sent
    ///
    /// # Params
    /// * frames - List of frames to be transmitted
    ///
    /// # Panics
    /// * Panics if the devices CAN channel is not yet configured
    pub fn send_can_packets_respond(&self, frames: &[CanFrame], max_timeout_ms: u32) -> Result<()> {
        let channel_id = match self.can_channel.as_ref() {
            Some(channel) => channel.id,
            None => panic!("CAN Channel has not been configured!"),
        };
        let mut msgs: Vec<PASSTHRU_MSG> = frames.iter().map(|f| f.to_pt_msg()).collect();
        self.device_driver
            .write_messages(channel_id, &mut msgs, max_timeout_ms)
            .map_err(|e| self.convert_error(e))
            .map(|_| ())
    }

    /// Attempts to get CAN Frames from the adapter that are queued in its Rx buffer.
    ///
    /// for the adapter to listen, you must configure at least one of its Rx filters for the CAN channel
    ///
    /// # Panics
    /// * Panics if the devices CAN channel is not yet configured
    pub fn receive_frames(&self, max_msgs: u32, max_timeout_ms: u32) -> Result<Vec<CanFrame>> {
        let channel_id = match self.can_channel.as_ref() {
            Some(channel) => channel.id,
            None => panic!("CAN Channel has not been configured!"),
        };
        self.device_driver
            .read_messages(channel_id, max_msgs, max_timeout_ms)
            .map_err(|e| self.convert_error(e))
            .map(|res| {
                res.iter()
                    .map(|f| {
                        CanFrame::from_byte_array(
                            &f.data[0..f.data_size as usize],
                            f.rx_status & RxFlag::CAN_29BIT_ID.bits() > 0,
                        )
                    })
                    .collect()
            })
    }

    /// Adds a CAN Filter to the device configured as a block filter
    ///
    /// # Returns
    /// Returns the filter ID from the device
    ///
    /// # Panics
    /// * Panics if the devices CAN channel is not yet configured
    pub fn add_can_filter_block(&self, pattern: u32, mask: u32) -> Result<u32> {
        let channel_id = match self.can_channel.as_ref() {
            Some(channel) => channel.id,
            None => panic!("CAN Channel has not been configured!"),
        };
        let mut m = PASSTHRU_MSG::default(); // Mask message
        let mut p = PASSTHRU_MSG::default(); // Pattern message
        m.data_size = 4;
        p.data_size = 4;

        m.protocol_id = Protocol::CAN as u32;
        p.protocol_id = Protocol::CAN as u32;

        m.data[0] = (mask >> 24) as u8;
        m.data[1] = (mask >> 16) as u8;
        m.data[2] = (mask >>  8) as u8;
        m.data[3] = (mask >>  0) as u8;

        p.data[0] = (pattern >> 24) as u8;
        p.data[1] = (pattern >> 16) as u8;
        p.data[2] = (pattern >>  8) as u8;
        p.data[3] = (pattern >>  0) as u8;

        self.device_driver.start_msg_filter(channel_id, J2534Common::FilterType::BLOCK_FILTER, &m, &p, None).map_err(|e| self.convert_error(e))
    }

    /// Adds a CAN Filter to the device configured as a pass filter
    ///
    /// # Panics
    /// * Panics if the devices CAN channel is not yet configured
    pub fn add_can_filter_pass(&self, pattern: u32, mask: u32) -> Result<u32> {
        let channel_id = match self.can_channel.as_ref() {
            Some(channel) => channel.id,
            None => panic!("CAN Channel has not been configured!"),
        };
        let mut m = PASSTHRU_MSG::default(); // Mask message
        let mut p = PASSTHRU_MSG::default(); // Pattern message
        m.data_size = 4;
        p.data_size = 4;

        m.protocol_id = Protocol::CAN as u32;
        p.protocol_id = Protocol::CAN as u32;

        m.data[0] = (mask >> 24) as u8;
        m.data[1] = (mask >> 16) as u8;
        m.data[2] = (mask >>  8) as u8;
        m.data[3] = (mask >>  0) as u8;

        p.data[0] = (pattern >> 24) as u8;
        p.data[1] = (pattern >> 16) as u8;
        p.data[2] = (pattern >>  8) as u8;
        p.data[3] = (pattern >>  0) as u8;

        self.device_driver.start_msg_filter(channel_id, J2534Common::FilterType::PASS_FILTER, &m, &p, None).map_err(|e| self.convert_error(e))
    
    }

    /// Attempts to remove a can filter applied to the device
    pub fn remove_can_filter(&self, filter_id: u32) -> Result<()> {
        if let Some(c) = self.can_channel {
            return self.device_driver.stop_msg_filter(c.id, filter_id).map_err(|e| self.convert_error(e))
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_connect() {

    }
}
