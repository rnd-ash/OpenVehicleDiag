use crate::commapi::protocols::{ProtocolResult, ProtocolServer};

use super::KWP2000ECU;

// The service, Start Diagnostic Session ($10), is used by the diagnostic tool to enable
// different types of diagnostic sessions in an ECU.  In order to execute a diagnostic
// service the appropriate session has to be started first. See Table 3.2.1-1 on page 8
// for a complete list of which service ID’s are supported by each diagnostic session.

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum DiagSession {
    Default = 0x81,
    Flash = 0x85,
    Standby = 0x89,
    Passive = 0x90,
    Extended = 0x92,
}

/// Attempts to set the diagnostic session type of the ECU
pub fn set_diag_session(ecu: &KWP2000ECU, mode: DiagSession) -> ProtocolResult<()> {
    ecu.run_command(super::Service::StartDiagSession.into(), &[mode as u8])?;
    Ok(())
}
