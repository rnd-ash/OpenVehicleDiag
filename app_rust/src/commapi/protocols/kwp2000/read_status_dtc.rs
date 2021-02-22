use crate::commapi::protocols::{ProtocolResult, ProtocolServer};

use super::KWP2000ECU;

/// Attempts to reset the ECU
pub fn read_status_dtc(ecu: &KWP2000ECU, dtc: u16) -> ProtocolResult<Vec<u8>> {
    ecu.run_command(
        super::Service::ReadDTCStatus.into(),
        &[(dtc >> 8) as u8, dtc as u8],
    )
}
