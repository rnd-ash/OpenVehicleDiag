use crate::commapi::protocols::{DTC, ProtocolResult, ProtocolServer};

use super::UDSECU;

pub fn read_dtc_information(ecu: &UDSECU, dtc: &DTC) -> ProtocolResult<Vec<u8>> {
    ecu.run_command(
        super::UDSCommand::ReadDTCInformation.into(),
        &[
            0x06, // reportDTCExtendedDataRecordByDTCNumber
            (dtc.id >> 16) as u8, // High byte
            (dtc.id >> 8) as u8, // Mid byte
            (dtc.id) as u8 // Low byte
        ],
    )
}