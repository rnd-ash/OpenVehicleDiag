

#[derive(Debug, Clone, Default)]
pub struct VirtualIface {

}

impl VirtualIface {
    fn log_msg(s: String) {
        println!("->IFACE: {}", s)
    }

    pub fn connect(&self) {
        Self::log_msg("Setting up virtual interface".into());
    }

    pub fn teardown(&self) {
        Self::log_msg("Tearing down old interface".into());
    }

    pub fn set_comm_repeats(&mut self, count: u32) {
        Self::log_msg(format!("Setting comm repeats to {}", count))
    }

    pub fn set_comm_params(&mut self, params: &[u32]) {
        Self::log_msg(format!("Setting comm params to {:08X?}", params))
    }

    pub fn set_answer_len(&mut self, len: &[u16]) {
        Self::log_msg(format!("Setting answer array length to {:04X?}", len))
    }

    pub fn send_receive_bytes(&mut self, out: &[u8]) -> Vec<u8> {
        Self::log_msg(format!("Sending bytes: {:02X?}", out));
        Vec::new()
    }
}