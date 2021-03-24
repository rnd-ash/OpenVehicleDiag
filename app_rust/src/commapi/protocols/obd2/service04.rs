use crate::commapi::protocols::ProtocolServer;

use super::{OBDError, ObdServer};


pub struct Service04;

impl Service04 {
    pub fn clear_dtcs(s: &ObdServer) -> OBDError<()> {
        s.run_command(0x04, &[]).map(|_| ())
    }
}