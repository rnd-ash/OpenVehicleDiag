use serde::{Serialize, Deserialize};

// Contains common data types and structures


#[derive(Serialize, Deserialize, Debug)]
/// ECU Type
struct ECU {
    name: String,
    protocol: Protocol,
}


#[derive(Serialize, Deserialize, Debug)]
enum Protocol {
    ISO15765,
    ISO9141,
    CAN,
}