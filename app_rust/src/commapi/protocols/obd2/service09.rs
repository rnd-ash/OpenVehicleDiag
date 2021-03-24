use crate::commapi::protocols::{ProtocolError, ProtocolServer};

use super::{OBDError, ObdError, ObdServer, ObdService, get_obd_bits};


pub struct Service09{
    supported_pids: Vec<bool>   
}

impl ObdService for Service09 {
    fn init(s: &ObdServer) -> Option<Self> {
        println!("Attempt init service 09!");
        let res = s.run_command(0x09, &[0x00]).ok()?;
        println!("Service 09 init bytes: {:02X?}", &res[2..]); // Drop SID and CID
        let bits = get_obd_bits(&res[2..]);
        let s09 = Service09{
            supported_pids: bits
        };
        if let Ok(vin) = s09.get_vin(s) {
            println!("VIN: {}", vin);
        }
        if let Ok(vin) = s09.get_calibration_id(s) {
            println!("CID: {}", vin);
        }

        if let Ok(cvn) = s09.get_calibration_verification_numbers(s) {
            println!("CVN's: {:?}", cvn);
        }

        Some(s09)
    }
}

impl Service09 {
    fn check_service_supported(&self, pid: u8) -> OBDError<()> {
        if let Some(r) = self.supported_pids.get(pid as usize - 1) { // -1 as pid 0x00 is not here
            match r {
                true => Ok(()),
                false => Err(ProtocolError::ProtocolError(Box::new(ObdError::CmdNotSupported)))
            }
        } else {
            eprintln!("Warning. Out of range PID requested {:02X}", pid);
            Err(ProtocolError::ProtocolError(Box::new(ObdError::CmdNotSupported)))
        }
    }


    pub fn get_vin_msg_count(&self, s: &ObdServer) -> OBDError<u8> {
        self.check_service_supported(0x01)?;
        s.run_command(0x09, &[0x01]).map(|s| s[2])
    }

    pub fn get_vin(&self, s: &ObdServer) -> OBDError<String> {
        self.check_service_supported(0x02)?;
        s.run_command(0x09, &[0x02]).map(|s| String::from_utf8_lossy(&s[2..]).to_string())
    }

    pub fn get_calibration_id(&self, s: &ObdServer) -> OBDError<String> {
        self.check_service_supported(0x04)?;
        s.run_command(0x09, &[0x04]).map(|s| String::from_utf8_lossy(&s[2..]).to_string())
    }

    pub fn get_calibration_verification_numbers(&self, s: &ObdServer) -> OBDError<Vec<String>> {
        self.check_service_supported(0x06)?;
        let mut raw = s.run_command(0x09, &[0x06])?;
        raw.drain(0..2);
        let count = raw[0]; // Number of CVNs
        raw.drain(0..1);
        let mut res = Vec::new();
        if raw.len() == (count*4) as usize {
            // Valid number of bytes
            for _ in 0..count {
                // CVN format
                res.push(format!("{:02X}{:02X}{:02X}{:02X}", raw[0], raw[1], raw[2], raw[3]));
                raw.drain(0..4);
            }
            Ok(res)
        } else {
            eprintln!("Invaliud count!");
            Err(ProtocolError::InvalidResponseSize{ expect: 3 + (count*4) as usize, actual:  raw.len() + 2 })
        }
    }
}