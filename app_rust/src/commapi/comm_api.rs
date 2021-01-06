use std::result::Result;
use std::cmp::min;
use serde::export::Formatter;
use std::sync::Arc;
use std::fmt::Debug;

#[derive(Debug, Copy, Clone, Default)]
pub struct CanFrame {
    pub id: u32,
    pub dlc: u8,
    data: [u8; 8]
}

impl CanFrame {
    pub fn get_data(&self) -> &[u8] {
        return &self.data[0..self.dlc as usize];
    }
    pub fn new(id: u32, data: &[u8]) -> Self {
        let dlc = min(data.len(), 8) as usize;
        let mut can_data: [u8; 8] = [0; 8];
        can_data[0..dlc].copy_from_slice(&data[0..dlc]);
        Self {
            id,
            dlc: dlc as u8,
            data: can_data
        }
    }
}
unsafe impl Send for CanFrame{}
unsafe impl Sync for CanFrame{}

impl std::fmt::Display for CanFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: 0x{:04X} Data: {:02X?}", self.id, &self.data[0..self.dlc as usize])
    }
}



#[derive(Debug, Clone)]
pub struct ISO15765Data {
    pub(crate) id: u32,
    pub(crate) data: Vec<u8>,
    pub(crate) pad_frame: bool
}

unsafe impl Send for ISO15765Data{}
unsafe impl Sync for ISO15765Data{}


#[derive(Debug, Copy, Clone)]
pub enum FilterType {
    Pass,
    Block
}

#[derive(Debug, Clone)]
pub struct ComServerError {
    pub err_code: u32,
    pub err_desc: String
}

impl std::fmt::Display for ComServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error code {} ({})", self.err_code, self.err_desc)
    }
}

#[derive(Debug, Copy, Clone, Eq, Ord, PartialOrd, PartialEq)]
pub enum Capability {
    // The device supports the capability
    Yes,
    // The device does not support the capability
    No,
    // The API the device uses does not support the capability
    NA
}

impl Capability {
    pub (crate) fn from_bool(b: bool) -> Self {
        if b { Capability::Yes } else { Capability::No }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceCapabilities {
    pub (crate) name: String,
    pub (crate) vendor: String,
    pub (crate) library_path: String,
    pub (crate) device_fw_version: String,
    pub (crate) library_version: String,

    /// Supports J1850VPW
    pub (crate) j1850vpw: Capability,
    /// Supports J1850PWM
    pub (crate) j1850pwm: Capability,
    /// Supports regular CAN
    pub (crate) can: Capability,
    /// Supports ISO15765 (ISO-TP)
    pub (crate) iso15765: Capability,
    /// Supports K-Line ODB ISO9141
    pub (crate) iso9141: Capability,
    /// Supports K-Line KWP2000 ISO14230
    pub (crate) iso14230: Capability,
    /// Supports Ethernet DoIP
    pub (crate) ip: Capability,
}

impl DeviceCapabilities {
    pub fn get_name(&self) -> String { self.name.clone() }
    pub fn get_vendor(&self) -> String { self.vendor.clone() }
    pub fn get_lib_path(&self) -> String { self.library_path.clone() }

    pub fn support_can_fd(&self) -> Capability { self.can }
    pub fn supports_iso15765(&self) -> Capability { self.iso15765 }

    pub fn supports_j1850pwm(&self) -> Capability { self.j1850pwm }
    pub fn supports_j1850vpw(&self) -> Capability { self.j1850vpw }

    pub fn supports_iso9141(&self) -> Capability { self.iso9141 }
    pub fn supports_iso14230(&self) -> Capability { self.iso14230 }
    pub fn supports_doip(&self) -> Capability { self.ip }

    pub fn get_device_fw_version(&self) -> String { self.device_fw_version.clone() }
    pub fn get_library_version(&self) -> String { self.library_version.clone() }
}


pub trait ComServer : Send + Sync + Debug {
    /// Attempts to open and connect to the device
    fn open_device(&mut self) -> Result<(), ComServerError>;

    /// Closes the driver connection to the device. Once this function is ran,
    /// commands can no longer be issued to the adapter until [open_device](fn@open_device) is ran.
    fn close_device(&mut self) -> Result<(), ComServerError>;

    /// Attempts to send a list [can frames](CanFrame) to the vehicle's can network.
    ///
    /// ## Params
    /// * data - List of CAN Frames to send to the vehicle
    /// * timeout_ms - Timeout for waiting for conformation from the adapter. A value of 0
    ///                will tell the adapter to queue to messages and return instantly, meaning
    ///                no conformation is provided
    ///
    /// ## Returns
    /// The number of CAN Frames successfully written to the vehicle, if Timeout is 0, this
    /// number will always be equal to the number of frames that were provided.
    fn send_can_packets(&self, data: &[CanFrame], timeout_ms: u32) -> Result<usize, ComServerError>;

    /// Attempts to read a list of [can frames](CanFrame) from the vehicle's can network.
    ///
    /// *NOTE*: You must set a filter prior to using this function, or no data will ever be read.
    ///
    /// ## Params
    /// * timeout_ms - Timeout for waiting for data from the vehicle. A value of 0 tells the adapter
    /// to return whatever data it has in its Rx queue, and don't wait for any more
    ///
    /// * max_msgs - The maximum number of messages to read from the adapter.
    fn read_can_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<CanFrame>, ComServerError>;

    /// Sends a list of ISO-TP (ISO15765) payloads to a vehicles Canbus network
    ///
    /// NOTE: You must set the flow control filter (Response ID) and configure the block size
    /// and separation time with [set_iso15765_params](fn@set_iso15765_params) prior to sending messages.
    ///
    /// You can query responses from the ECU with [read_iso15765_packets](fn@read_iso15765_packets)
    ///
    /// ## Params
    /// * `data` - List of ISO-TP messages to send to the vehicle
    /// * `timeout_ms` - Timeout for waiting for conformation from the adapter. A value of 0
    ///                will tell the adapter to queue to messages and return instantly, meaning
    ///                no conformation is provided
    ///
    /// ## Returns
    /// The number of ISO-TP messages successfully written to the vehicle, if Timeout is 0, this
    /// number will always be equal to the number of frames that were provided.
    fn send_iso15765_data(&self, data: &[ISO15765Data], timeout_ms: u32) -> Result<usize, ComServerError>;

    /// Attempts to read a list of [iso-tp messages](ISO15765Data) from the vehicle's can network.
    ///
    /// *NOTE*: You must set a filter prior to using this function, or no data will ever be read.
    ///
    /// ## Params
    /// * timeout_ms - Timeout for waiting for data from the vehicle. A value of 0 tells the adapter
    /// to return whatever data it has in its Rx queue, and don't wait for any more
    ///
    /// * max_msgs - The maximum number of messages to read from the adapter.
    fn read_iso15765_packets(&self, timeout_ms: u32, max_msgs: usize) -> Result<Vec<ISO15765Data>, ComServerError>;

    /// Attempts to open a CAN interface with the adapter to the vehicles ODB-II port
    ///
    /// ## Params
    /// * `bus_speed` - Speed of the vehicle Canbus in bps, typically for an ODB-II port it is 500000
    /// * `is_ext_can` - Tells the adapter to use extended CAN Addressing
    fn open_can_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError>;

    /// Attempts to destroy the CAN Interface on the adapter
    fn close_can_interface(&mut self) -> Result<(), ComServerError>;

    /// Attempts to open a ISO-TP interface over CAN with the adapter to the vehicles ODB-II port
    ///
    /// ## Params
    /// * `bus_speed` - Speed of the vehicle Canbus in bps, typically for an ODB-II port it is 500000
    /// * `is_ext_can` - Tells the adapter to use extended CAN Addressing
    fn open_iso15765_interface(&mut self, bus_speed: u32, is_ext_can: bool) -> Result<(), ComServerError>;

    /// Attempts to destroy the ISO-TP Interface on the adapter
    fn close_iso15765_interface(&mut self) -> Result<(), ComServerError>;

    /// Attempts to create a new CAN Filter on the adapter, given an ID and Mask
    ///
    /// ## Params
    /// * filter - Filter type. Can either be configured to be a pass or block filter.
    /// A pass filter allows data that matches the mask and pattern ID to be received by the PC,
    /// where as a block filter prevents data that matches the pattern ID and mask from being
    /// received by the PC
    ///
    /// * id - CAN ID for pattern matching
    /// * mask - Mask ID for pattern matching
    ///
    /// ## Returns
    /// The filter ID provided by the adapter. Use this when destroying the filter
    fn add_can_filter(&self, filter: FilterType, id: u32, mask: u32) -> Result<u32, ComServerError>;

    /// Tells the adapter to remove an active filter on an open CAN channel
    /// # Params
    /// * filter_idx - Filter ID to remove, this should be the value given by [`add_can_filter`](fn@add_can_filter)
    fn rem_can_filter(&self, filter_idx: u32) -> Result<(), ComServerError>;

    fn add_iso15765_filter(&self, id: u32, mask: u32, resp_id: u32) -> Result<u32, ComServerError>;

    /// Tells the adapter to remove an active filter on an open ISO15765 channel
    /// # Params
    /// * filter_idx - Filter ID to remove, this should be the value given by [`add_iso15765_filter`](fn@add_iso15765_filter)
    fn rem_iso15765_filter(&self, filter_idx: u32) -> Result<(), ComServerError>;

    /// Tells the adapter to set the block size and separation time on an active
    /// ISO15765 channel
    ///
    /// If an ISO15765 channel is not currently opened on the device, this function
    /// will return an error.
    ///
    /// # Params
    /// * separation_time_min - The minimum separation time between sending frames to the ECU
    /// * block_size - The number of CAN frames to receive or send before waiting for another
    ///                 flow control message from the ECU
    fn set_iso15765_params(&self, separation_time_min: u32, block_size: u32) -> Result<(), ComServerError>;

    /// Tells the adapter to clear any data in its Rx buffer
    /// that is from CAN protocol
    fn clear_can_rx_buffer(&self) -> Result<(), ComServerError>;

    /// Tells the adapter to clear any data in its Tx buffer
    /// that is from CAN protocol
    fn clear_can_tx_buffer(&self) -> Result<(), ComServerError>;

    /// Tells the adapter to clear any data in its Rx buffer
    /// that is from the ISO15765 protocol
    fn clear_iso15765_rx_buffer(&self) -> Result<(), ComServerError>;

    /// Tells the adapter to clear any data in its Tx buffer
    /// that is from the ISO15765 protocol
    fn clear_iso15765_tx_buffer(&self) -> Result<(), ComServerError>;

    /// Returns the voltage read by the adapter on the +12V line of the ODB-II
    /// adapter, which is normally connected to the car battery
    ///
    /// # Returns
    /// * Returns the voltage in Volts
    fn read_battery_voltage(&self) -> Result<f32, ComServerError>;

    /// Clones this in memory into a new Box
    fn clone_box(&self) -> Box::<dyn ComServer>;

    /// Retrieves the device's capabilities
    fn get_capabilities(&self) -> DeviceCapabilities;

    /// Returns a 1 word string indicating which hardware API the device uses
    fn get_api(&self) -> &str;
}

impl Clone for Box<dyn ComServer> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}