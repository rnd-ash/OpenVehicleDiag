use serde::{Deserialize, Serialize};
use J2534Common::Protocol;
use serde_json::*;
/// Schema V1 for data contains that OVD uses
#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaV1 {
    meta: ECUMeta,
    err_table: Vec<DTC>,
    comm_data: Vec<CommData>
}

impl SchemaV1 {
    pub fn to_file(&self, path: &str) {
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).unwrap();
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TesterPresent {
    sid: u32,
    rid: u32,
    spayload: Vec<u8>
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CommData {
    protocol: Protocol,
    baud: u32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Function {

}

#[derive(Debug, Serialize, Deserialize)]
pub struct DTC {
    /// Name of error code. Example: P2000
    name: String,
    /// Description of error code
    desc: String
}

#[derive(Debug, Serialize, Deserialize)]
/// ECU Metadata
pub struct ECUMeta {
    /// Name of ECU. Example EGS52
    name: String,
    /// ECU Vendor name
    vendor: String,
    /// Simple description of the ECU
    desc: String,
}



#[test]
/// Simple test to create a demo of EGS52 control unit
fn create_egs52() {
    let meta = ECUMeta {
        name: "EGS52".to_string(),
        vendor: "Mercedes-Benz".to_string(),
        desc: "722.6 Controller Generation 2".to_string()
    };
    println!("{:#?}", meta);
}