use crate::commapi::protocols::{DTC, ProtocolServer};

use super::{OBDError, ObdServer};




pub struct Service03;

impl Service03 {
    pub fn read_dtcs(s: &ObdServer) -> OBDError<Vec<DTC>> {
        let res = s.run_command(0x03, &[])?;
        println!("Res: {:02X?}", res);
        // TODO 
        Ok(Vec::new())
    }
}