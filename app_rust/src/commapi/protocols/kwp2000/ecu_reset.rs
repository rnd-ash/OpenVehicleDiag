use crate::commapi::protocols::{ProtocolResult, ProtocolServer};

use super::KWP2000ECU;



/*
The service, ECU Reset ($11), requests an ECU to effectively 
perform a reset based on the content of the Reset Mode parameter.  
The Reset Mode parameter may specify a reset for the entire ECU 
or selective portions of on-board memory. 
*/


#[derive(Debug, Copy, Clone)]
pub enum ResetType {
    PowerOnReset = 0x01,
    MemoryReset = 0x82
}

/// Attempts to reset the ECU
pub fn reset_ecu(ecu: &KWP2000ECU, reset_type: ResetType) -> ProtocolResult<()> {
    ecu.run_command(super::Service::ECUReset.into(), &[reset_type as u8])?;
    Ok(())
}