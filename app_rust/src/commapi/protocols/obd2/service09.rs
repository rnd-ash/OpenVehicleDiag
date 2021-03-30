use crate::commapi::protocols::{ProtocolError, ProtocolServer};

use super::{OBDError, ObdError, ObdServer, ObdService, get_obd_bits};


#[derive(Debug, Clone)]
pub struct Service09{
    supported_pids: Vec<bool> 
}

#[derive(Debug, Clone, Default)]
pub struct Service09Data {
    pub vin: String,
    pub calibration_id: String,
    pub cvns: Vec<String>,
    pub ecu_name: String
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

    pub fn get_everything(&self, s: &ObdServer) -> Service09Data {
        Service09Data {
            vin: self.get_vin(s).unwrap_or("Not Supported".into()),
            calibration_id: self.get_calibration_id(s).unwrap_or("Not Supported".into()),
            cvns: self.get_calibration_verification_numbers(s).unwrap_or_default(),
            ecu_name: self.get_ecu_name(s).unwrap_or("Not Supported".into()),

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

    pub fn get_ecu_name(&self, s: &ObdServer) -> OBDError<String> {
        self.check_service_supported(0x0A)?;
        s.run_command(0x09, &[0x0A]).map(|s| String::from_utf8_lossy(&s[2..]).to_string())
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