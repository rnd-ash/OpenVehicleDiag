use crate::commapi::protocols::{ProtocolResult, ProtocolServer};

use super::UDSECU;

// The service, Start Diagnostic Session ($10), is used by the diagnostic tool to enable
// different types of diagnostic sessions in an ECU.  In order to execute a diagnostic
// service the appropriate session has to be started first. See Table 3.2.1-1 on page 8
// for a complete list of which service ID’s are supported by each diagnostic session.

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum DiagSession {
    Default,
    Programming,
    Extended,
    SafetySystem,
    VehicleSpecific(u8),
    SystemSupplier(u8),
}

impl DiagSession {
    fn to_byte(&self) -> u8 {
        match &self {
            DiagSession::Default => 0x01,
            DiagSession::Programming => 0x02,
            DiagSession::Extended => 0x03,
            DiagSession::SafetySystem => 0x04,
            DiagSession::VehicleSpecific(x) => *x,
            DiagSession::SystemSupplier(x) => *x,
        }
    }
}

/// Attempts to set the diagnostic session type of the ECU
pub fn set_diag_session(ecu: &UDSECU, mode: DiagSession) -> ProtocolResult<()> {
    ecu.run_command(
        super::UDSCommand::DiagnosticSessionControl.into(),
        &[mode.to_byte()],
    )?;
    Ok(())
}
