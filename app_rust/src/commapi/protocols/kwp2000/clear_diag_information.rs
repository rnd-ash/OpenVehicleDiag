use crate::commapi::protocols::{ProtocolResult, ProtocolServer};

use super::KWP2000ECU;



/*
The service, Clear Diagnostic Information ($14), is used by the 
diagnostic tool to clear Diagnostic Trouble Codes (DTC) and 
associated diagnostic information in the ECU’s memory. 
*/

#[derive(Debug, Copy, Clone)]
pub enum DTCGroup {
    Powertrain = 0x0000,
    Chassis = 0x4000,
    Body = 0x8000,
    Network = 0xC000,
    All = 0xFF00
}

pub fn clear_custom_dtc(ecu: &KWP2000ECU, code: u16) -> ProtocolResult<()> {
    ecu.run_command(super::Service::ClearDiagnosticInformation.into(), &[(code >> 8) as u8, code as u8])?;
    Ok(())
}

pub fn clear_dtc_group(ecu: &KWP2000ECU, group: DTCGroup) -> ProtocolResult<()> {
    ecu.run_command(super::Service::ClearDiagnosticInformation.into(), &[(group as u16 >> 8) as u8, group as u8])?;
    Ok(())
}