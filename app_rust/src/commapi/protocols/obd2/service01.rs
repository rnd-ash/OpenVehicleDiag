use std::{cmp::min, sync::Arc, vec};

use lazy_static::lazy_static;

use crate::commapi::protocols::{ProtocolError, ProtocolServer};

use super::{OBDError, ObdError, ObdServer, ObdService, get_obd_bits};

lazy_static! {
    static ref PID_LIST: PidList = PidList::init_list();
}

#[derive(Clone)]
struct PidConvert {
    desc: &'static str,
    unit: &'static str,
    func: Arc<Box<dyn Fn(f32, f32, f32, f32) -> f32>>,
    bounds: (f32, f32)
}

unsafe impl Send for PidConvert{}
unsafe impl Sync for PidConvert{}

impl PidConvert {
    fn parse(&self, args: [u8; 4]) -> f32 {
        (self.func)(args[0] as f32, args[1] as f32, args[2] as f32, args[3] as f32)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PidResult<'a> {
    desc: &'a str,
    unit: &'a str,
    res: f32,
    bounds: (f32, f32)
}

pub struct PidList {
    pids: Vec<Option<PidConvert>>
}

impl PidList {
    pub (crate) fn init_list() -> Self {
        let mut res = PidList { pids: vec![None; 0xFF] };
        res.add_func(0x04, "Calculated engine load", "%", (0f32, 100f32), &|a, _, _, _|{ a/2.55f32 });
        res
    }

    pub fn add_func<F: Fn(f32, f32, f32, f32) -> f32>(&mut self, pid: u8, desc: &'static str, unit: &'static str, bounds: (f32, f32), f: &'static F) {
        self.pids[pid as usize] = Some(PidConvert {
            desc: desc,
            unit: unit,
            func: Arc::new(Box::new(f)),
            bounds,
        });
    }

    pub fn parse_pid(&self, pid: u8, args: &[u8]) -> Option<PidResult> {
        let parser = self.pids[pid as usize].as_ref()?;
        let len = min(4, args.len());
        let mut n : [u8; 4] = [0x00; 4];
        n[0..len].copy_from_slice(&args[0..len]);
        Some(PidResult {
            desc: parser.desc,
            unit: parser.unit,
            res: parser.parse(n),
            bounds: parser.bounds,
        })
    }
}

pub struct Service01 {
    supported_pids: Vec<bool>
}

impl ObdService for Service01 {
    fn init(s: &ObdServer) -> Option<Self> {
        println!("Attempt init service 01!");
        println!("Check PIDS 01-20");
        let mut res = s.run_command(0x01, &[0x00]).ok()?;
        let mut s01 = Service01 { supported_pids: get_obd_bits(&res[2..]) };

        // Now query extra 01 pids
        if s01.check_service_supported(0x20).is_ok() {
            println!("Check PIDS 21-40");
            // Check 21-40
            res = s.run_command(0x01, &[0x20]).ok()?;
            s01.supported_pids.append(&mut get_obd_bits(&res[2..]));
        }
        if s01.check_service_supported(0x40).is_ok() {
            println!("Check PIDS 41-60");
            // Check 41-60
            res = s.run_command(0x01, &[0x40]).ok()?;
            s01.supported_pids.append(&mut get_obd_bits(&res[2..]));
        }
        if s01.check_service_supported(0x60).is_ok() {
            println!("Check PIDS 61-80");
            // Check 61-80
            res = s.run_command(0x01, &[0x60]).ok()?;
            s01.supported_pids.append(&mut get_obd_bits(&res[2..]));
        }
        if s01.check_service_supported(0x80).is_ok() {
            println!("Check PIDS 81-A0");
            // Check 81-A0
            res = s.run_command(0x01, &[0x80]).ok()?;
            s01.supported_pids.append(&mut get_obd_bits(&res[2..]));
        }
        if s01.check_service_supported(0xA0).is_ok() {
            println!("Check PIDS A1-C0");
            // Check A1-C0
            res = s.run_command(0x01, &[0xA0]).ok()?;
            s01.supported_pids.append(&mut get_obd_bits(&res[2..]));
        }
        if s01.check_service_supported(0xC0).is_ok() {
            println!("Check PIDS C1-E0");
            // Check C1-E0
            res = s.run_command(0x01, &[0xC0]).ok()?;
            s01.supported_pids.append(&mut get_obd_bits(&res[2..]));
        }
        Some(s01)
    }
}

impl Service01 {
    fn check_service_supported(&self, pid: u8) -> OBDError<()> {
        if let Some(r) = self.supported_pids.get(pid as usize - 1) { // -1 as pid 0x00 is not here
            match r {
                true => Ok(()),
                false => Err(ProtocolError::ProtocolError(Box::new(ObdError::CmdNotSupported)))
            }
        } else {
            Err(ProtocolError::ProtocolError(Box::new(ObdError::CmdNotSupported)))
        }
    }

    pub fn get_chartable_pid(&self, s: &ObdServer, pid: u8) -> OBDError<Option<PidResult>> {
        self.check_service_supported(pid)?;
        let bytes = s.run_command(0x01, &[pid])?;
        Ok(PID_LIST.parse_pid(pid, &bytes[2..]))
    }
}