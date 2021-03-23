pub mod variant;
use serde::{Serialize, Deserialize};
use variant::ECUVariantDefinition;
pub mod diag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OvdECU {
    pub name: String,
    pub description: String,
    pub variants: Vec<ECUVariantDefinition>,
    // ECU Can have multiple connection types (KLINE, ISOTP)
    pub connections: Vec<Connection>
}
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Defines connection properties on how to communicate with the ECU
pub struct Connection {
    /// Protocol baud rate
    pub baud: u32,
    /// Send ID for sending data to the ECU
    pub send_id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "Option::default")]
    /// Optional global request ID for Tester present.
    /// MB uses this with interior CAN Devices
    pub global_send_id: Option<u32>,
    /// Type of connection to use
    pub connection_type: ConType,
    /// Which diagnostic protocol to use
    pub server_type: ServerType,
    /// Receive ID for receiving data from the ECU
    pub recv_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Diagnostic server enumeration
pub enum ServerType {
    /// UDS diagnostic server
    UDS,
    /// KWP2000 diagnostic server
    KWP2000
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConType {
    /// LIN (K-LINE) Connection type
    LIN {
        /// Max data segment size
        max_segment_size: u32,
        /// Wake up method for waking up K-Line
        wake_up_method: LinWakeUpType
    },
    /// ISO-TP Over CAN connection type
    ISOTP { 
        /// ISO-TP block size 
        blocksize: u32,
        /// ISO-TP minimum packet separation time
        st_min: u32 
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// K-Line wake up method
pub enum LinWakeUpType {
    /// 5 baud initialization
    /// The tester will send the tester address over the bus at 5bps to wake up the host ECU
    FiveBaudInit,
    /// Fast initialization
    /// The tester will sent a wakeup pattern at 10400bps to wake up the bus
    FastInit
}