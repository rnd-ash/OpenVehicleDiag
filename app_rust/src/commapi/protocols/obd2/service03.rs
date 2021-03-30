use crate::commapi::protocols::{DTC, ProtocolServer, DTCState};

use super::{OBDError, ObdServer};



#[derive(Debug, Clone)]
pub struct Service03;

impl Service03 {
    pub fn read_dtcs(s: &ObdServer) -> OBDError<Vec<DTC>> {
        let mut bytes = s.run_command(0x03, &[])?;
        let num_dtcs = bytes[1];
        bytes.drain(0..2);
        let mut res = Vec::new();
        for _ in 0..num_dtcs {
            let prefix = match (bytes[0] >> 6) & 0b0000011 {
                1 => 'C', // Chassis
                2 => 'B', // Body
                3 => 'U', // Network
                _ => 'P'  // Powertrain
            };
            let n1 = (bytes[0] >> 4) & 0b0000011;
            let n2 = (bytes[0] >> 2) & 0b0000011;
            let dtc = DTC {
                error: format!("{}{:1X}{:1X}{:2X}", prefix, n1, n2, bytes[1]),
                state: DTCState::Stored, // TODO Fix this
                check_engine_on: true,
                
            };
            bytes.drain(0..2);
            res.push(dtc);
        }
        // TODO 
        Ok(res)
    }
}