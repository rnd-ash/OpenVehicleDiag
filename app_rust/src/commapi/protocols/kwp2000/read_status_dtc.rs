use crate::commapi::protocols::{DTC, ProtocolResult, ProtocolServer};

use super::KWP2000ECU;

/// Attempts to reset envs from DTC
pub fn read_status_dtc(ecu: &KWP2000ECU, dtc: &DTC) -> ProtocolResult<Vec<u8>> {
    ecu.run_command(
        super::Service::ReadDTCStatus.into(),
        &[(dtc.id >> 8) as u8, dtc.id as u8],
    )
}
