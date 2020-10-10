use serde::{Serialize, Deserialize};
pub mod uds;

// Contains common data types and structures


#[derive(Serialize, Deserialize, Debug)]
/// ECU Type
struct ECU {
    pub name: String,
    pub protocol: Protocol,
}


#[derive(Serialize, Deserialize, Debug)]
pub enum Protocol {
    /// Multi packet CAN Frames
    ISO15765,
    /// Standard K-Line ODB2
    ISO9141,
    /// Single CAN Frames
    CAN,
}

enum SecurityLevel {
    ALWAYS = 0,
    TESTER_PRESENT = 1,
    KEY_REQUIRED = 2
}


pub struct UDSPayload {
    pub id: uds::UDSCommandID,
    pub args: Vec<u8>
}

struct Function {
    name: String,
    //securityLevel: SecurityLevel
    payload: Vec<u8>
}

struct DTC {
    name: String,
    desc: String
}