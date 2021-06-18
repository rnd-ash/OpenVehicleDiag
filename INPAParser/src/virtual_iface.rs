

#[derive(Debug, Clone, Default)]
pub struct VirtualIface {
    comm_repeats_protected: u32,
    comm_params_protected: Vec<u32>,
    comm_answer_len_protected: Vec<u16>,
    comm_parameter: Vec<u32>
}

impl VirtualIface {

    pub fn new() -> Self {
        let mut x = Self::default();
        x.comm_answer_len_protected = vec![0x00; 2];
        x
    }

    fn log_msg(s: String) {
        println!("->IFACE: {}", s)
    }

    pub fn connect(&mut self) {
        Self::log_msg("Setting up virtual interface".into());
        self.comm_repeats_protected = 0;
        self.comm_params_protected.clear();
        self.comm_answer_len_protected[0] = 0x00;
        self.comm_answer_len_protected[1] = 0x00;
    }

    pub fn teardown(&mut self) {
        Self::log_msg("Tearing down old interface".into());
    }

    pub fn set_comm_repeats(&mut self, count: u32) {
        Self::log_msg(format!("Setting comm repeats to {}", count));
        self.comm_repeats_protected = count;
    }

    pub fn set_comm_params(&mut self, params: &[u32]) {
        Self::log_msg(format!("Setting comm params to {:08X?}", params));
        self.comm_params_protected.clear();
        self.comm_params_protected.extend_from_slice(params);

        self.comm_answer_len_protected[0] = 0;
        self.comm_answer_len_protected[1] = 0;

        let concept = self.comm_params_protected[0];
        match concept {
            0x0000 => { 
                match self.comm_params_protected[4] {
                    0x81 => Self::log_msg("Communication protocol is Raw EDIC (KWP2000)".into()),
                    0xA4 => Self::log_msg("Communication protocol is Raw EDIC (KWP2000 TP1.6)".into()),
                    0xA5 => Self::log_msg("Communication protocol is Raw EDIC (KWP2000 TP2.0)".into()),
                    0xAA => Self::log_msg("Communication protocol is Raw EDIC (KWP2000 ISO-TP)".into()),
                    _ => Self::log_msg("Communication protocol is Raw EDIC (UNSUPPORTED)".into())
                }
             },
            0x0001 => { Self::log_msg("Communication protocol is Concept 1".into()) },
            0x0002 => { Self::log_msg("Communication protocol is ISO9141 KWP1281".into()) },
            0x0003 => { Self::log_msg("Communication protocol is Concept 3".into()) },
            0x0005 | 0x0006 => { Self::log_msg("Communication protocol is DS1/2".into()) },
            0x010C => { Self::log_msg("Communication protocol is KWP2000 (BMW)".into()) },
            0x010D => { Self::log_msg("Communication protocol is KWP2000".into()) },
            0x010F => { Self::log_msg("Communication protocol is BMW-FAST".into()) },
            0x110 => { Self::log_msg("Communication protocol is D-CAN".into()) },
            _ => {}
        }

    }

    pub fn set_answer_len(&mut self, len: &[u16]) {
        Self::log_msg(format!("Setting comm answer length to {:04X?}", len));
        if len.len() >= 2 {
            self.comm_answer_len_protected[0] = len[0];
            self.comm_answer_len_protected[1] = len[1];
        }
    }

    pub fn send_receive_bytes(&mut self, out: &[u8]) -> Vec<u8> {
        Self::log_msg(format!("Sending bytes: {:02X?}", out));
        Vec::new()
    }
}