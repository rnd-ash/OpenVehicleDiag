use std::{borrow::Borrow, cmp::min, sync::Arc, vec};

use lazy_static::lazy_static;

use crate::commapi::protocols::{ProtocolError, ProtocolServer};

use super::{OBDError, ObdError, ObdServer, ObdService, get_obd_bits};

lazy_static! {
    static ref PID_LIST: PidList = PidList::init_list();
}

#[derive(Debug, Copy, Clone)]
enum OBDDataType {
    Number,
    MultiNumber,
    String,
}

#[derive(Debug, Clone)]
pub enum PidReturnType<'a> {
    Number(PidResult<'a>),
    MultiNumber(Vec<PidResult<'a>>),
    String(String) // Enums
}

#[derive(Clone)]
struct PidConvert {
    data_type: OBDDataType,
    desc: Vec<&'static str>,
    unit: Vec<&'static str>,
    fun1: Arc<Box<dyn Fn(f32, f32, f32, f32) -> f32>>,
    fun2: Arc<Box<dyn Fn(f32, f32, f32, f32) -> Vec<f32>>>,
    fun3: Arc<Box<dyn Fn(u8, u8, u8, u8) -> String>>,
    bounds: Vec<(f32, f32)>
}

unsafe impl Send for PidConvert{}
unsafe impl Sync for PidConvert{}

impl PidConvert {
    fn parse(&self, args: [u8; 4]) -> PidReturnType {
        match self.data_type {
            OBDDataType::Number => PidReturnType::Number(PidResult{
                desc: self.desc[0],
                unit: self.unit[0],
                bounds: self.bounds[0],
                res: (self.fun1)(args[0] as f32, args[1] as f32, args[2] as f32, args[3] as f32)
            }),
            OBDDataType::MultiNumber => {
                let t = (self.fun2)(args[0] as f32, args[1] as f32, args[2] as f32, args[3] as f32);
                let mut res = Vec::new();
                for (idx, x) in t.iter().enumerate() {
                    res.push(
                        PidResult{
                            desc: self.desc[idx],
                            unit: self.unit[idx],
                            bounds: self.bounds[idx],
                            res: *x
                        }
                    )
                }

                PidReturnType::MultiNumber(res)
            },
            OBDDataType::String => PidReturnType::String((self.fun3)(args[0], args[1], args[2], args[3]))
        }
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
        // Add new PIDS here!
        res.add_func_str(0x03, "Fuel system status", &|a, _, _, _| { PidList::get_fuel_status(a) });
        res.add_func_num(0x04, "Calculated engine load", "%", (0.0, 100.0), &|a, _, _, _|{ a/2.55 });
        res.add_func_num(0x05, "Engine coolant tempurature", "\u{00B0}C", (-40.0, 215.0), &|a, _, _, _|{ a - 40.0 });
        res.add_func_num(0x06, "Short term fuel trim - Bank 1", "%", (-100.0, 99.2), &|a, _, _, _|{ a/1.28 - 100.0 });
        res.add_func_num(0x07, "Long term fuel trim - Bank 1", "%", (-100.0, 99.2), &|a, _, _, _|{ a/1.28 - 100.0 });
        res.add_func_num(0x08, "Short term fuel trim - Bank 2", "%", (-100.0, 99.2), &|a, _, _, _|{ a/1.28 - 100.0 });
        res.add_func_num(0x09, "Long term fuel trim - Bank 2", "%", (-100.0, 99.2), &|a, _, _, _|{ a/1.28 - 100.0 });
        res.add_func_num(0x0A, "Fuel pressure", "kPa", (0.0, 765.0), &|a, _, _, _|{ a * 3.0 });
        res.add_func_num(0x0B, "Intake manifold absolute pressure", "kPa", (0.0, 255.0), &|a, _, _, _|{ a });
        res.add_func_num(0x0C, "Engine speed", "rpm", (0.0, 16383.75), &|a, b, _, _|{ ((256.0 * a) + b)/4.0 });
        res.add_func_num(0x0D, "Vehicle speed", "km/h", (0.0, 255.0), &|a, _, _, _|{ a });
        res.add_func_num(0x0E, "Timing advance", "\u{00B0} before TDC", (-64.0, 63.5), &|a, _, _, _|{ a/2.0 - 64.0 });
        res.add_func_num(0x0F, "Intake air temperature", "\u{00B0}C", (-40.0, 215.0), &|a, _, _, _|{ a - 40.0 });
        res.add_func_num(0x10, "Mass air flow sensor (MAF)", "grams/sec", (0.0, 655.35), &|a, b, _, _|{ ((256.0 * a) + b) / 100.0 });
        res.add_func_num(0x11, "Throttle position", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_str(0x12, "Commanded secondary air status", &|a, _, _, _| { PidList::get_secondary_air_status(a) });
        res.add_func_mult(0x14, &["O2 sensor 1 voltage", "O2 sensor 1 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x15, &["O2 sensor 2 voltage", "O2 sensor 2 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x16, &["O2 sensor 3 voltage", "O2 sensor 3 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x17, &["O2 sensor 4 voltage", "O2 sensor 4 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x18, &["O2 sensor 5 voltage", "O2 sensor 5 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x19, &["O2 sensor 6 voltage", "O2 sensor 6 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x1A, &["O2 sensor 7 voltage", "O2 sensor 7 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_mult(0x1B, &["O2 sensor 8 voltage", "O2 sensor 8 short fuel trim"], &["V", "%"], &[(0.0, 1.275), (-100.0, 99.2)], &|a, b, _, _| { vec![a/200.0, (1.28*b) - 100.0] });
        res.add_func_str(0x1C, "OBD standards supported", &|a, _, _, _| { PidList::get_obd_standard(a) });
        res.add_func_str(0x1E, "AUX input status", &|a, _, _, _| {if a == 0 {"Power take off inactive"} else {"Power take off active"}.into() });
        res.add_func_num(0x1F, "Run time since engine start", "seconds", (0.0, 65535.0), &|a, b, _, _|{ (a * 256.0) + b });
        res.add_func_num(0x21, "Distance traveled with MIL on", "km", (0.0, 65535.0), &|a, b, _, _|{ (a * 256.0) + b });
        res.add_func_num(0x22, "Fuel rail pressure", "kPa", (0.0, 5177.265), &|a, b, _, _|{ 0.079 * ((a * 256.0) + b) });
        res.add_func_num(0x23, "Fuel rail gauge pressure", "kPa", (0.0, 655350.0), &|a, b, _, _|{ 10.0 * ((a * 256.0) + b) });
        res.add_func_mult(0x24, &["O2 sensor 1 Air-fuel ratio", "O2 sensor 1 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x25, &["O2 sensor 2 Air-fuel ratio", "O2 sensor 2 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x26, &["O2 sensor 3 Air-fuel ratio", "O2 sensor 3 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x27, &["O2 sensor 4 Air-fuel ratio", "O2 sensor 4 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x28, &["O2 sensor 5 Air-fuel ratio", "O2 sensor 5 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x29, &["O2 sensor 6 Air-fuel ratio", "O2 sensor 6 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x2A, &["O2 sensor 7 Air-fuel ratio", "O2 sensor 7 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_mult(0x2B, &["O2 sensor 8 Air-fuel ratio", "O2 sensor 8 voltage"], &["", "V"], &[(0.0, 2.0), (-100.0, 8.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), (8.0/65536.0)*((256.0*c)+d)] });
        res.add_func_num(0x2C, "Commanded EGR", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x2D, "EGR Error", "%", (-100.0, 99.2), &|a, _, _, _|{ (a / 1.28) - 100.0 });
        res.add_func_num(0x2E, "Commanded evaporative purge", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x2F, "Fuel tank level input", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x31, "Distance traveled since codes cleared", "km", (0.0, 65535.0), &|a, b, _, _|{ 0.079 * ((a * 256.0) + b) });
        res.add_func_num(0x32, "Evap. System vapor pressure", "Pa", (-8192.0, 8191.75), &|a, b, _, _|{ 0.25 * ((a * 256.0) + b) });
        res.add_func_num(0x33, "Absolute barometric pressure", "kPa", (0.0, 255.0), &|a, _, _, _|{ a });
        res.add_func_mult(0x34, &["O2 sensor 1 Air-fuel ratio", "O2 sensor 1 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x35, &["O2 sensor 2 Air-fuel ratio", "O2 sensor 2 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x36, &["O2 sensor 3 Air-fuel ratio", "O2 sensor 3 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x37, &["O2 sensor 4 Air-fuel ratio", "O2 sensor 4 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x38, &["O2 sensor 5 Air-fuel ratio", "O2 sensor 5 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x39, &["O2 sensor 6 Air-fuel ratio", "O2 sensor 6 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x3A, &["O2 sensor 7 Air-fuel ratio", "O2 sensor 7 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_mult(0x3B, &["O2 sensor 8 Air-fuel ratio", "O2 sensor 8 current"], &["", "mA"], &[(0.0, 2.0), (0.0, 128.0)], &|a, b, c, d| { vec![(2.0/65536.0)*((256.0*a)+b), c + (d / 256.0) - 128.0] });
        res.add_func_num(0x3C, "Catalyst temperature. Bank 1, Sensor 1", "\u{00B0}C", (-40.0, 6513.5), &|a, b, _, _|{ 0.1 * ((a * 256.0) + b) });
        res.add_func_num(0x3D, "Catalyst temperature. Bank 2, Sensor 1", "\u{00B0}C", (-40.0, 6513.5), &|a, b, _, _|{ 0.1 * ((a * 256.0) + b) });
        res.add_func_num(0x3E, "Catalyst temperature. Bank 1, Sensor 2", "\u{00B0}C", (-40.0, 6513.5), &|a, b, _, _|{ 0.1 * ((a * 256.0) + b) });
        res.add_func_num(0x3F, "Catalyst temperature. Bank 2, Sensor 2", "\u{00B0}C", (-40.0, 6513.5), &|a, b, _, _|{ 0.1 * ((a * 256.0) + b) });
        res.add_func_num(0x42, "Control module voltage", "V", (0.0, 65.535), &|a, b, _, _|{ 0.001 * ((a * 256.0) + b) });
        res.add_func_num(0x43, "Absolute load value", "%", (0.0, 25700.0), &|a, b, _, _|{ ((a * 256.0) + b) * (100.0/255.0) });
        res.add_func_num(0x44, "Commanded air-fuel ratio", "", (0.0, 2.0), &|a, b, _, _|{ ((a * 256.0) + b) * (2.0/65536.0) });
        res.add_func_num(0x45, "Relative throttle position", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x46, "Ambient air temperature", "\u{00B0}C", (-40.0, 215.0), &|a, _, _, _|{ a -40.0 });
        res.add_func_num(0x47, "Absolute throttle position B", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x48, "Absolute throttle position C", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x49, "Accelerator pedal position D", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x4A, "Accelerator pedal position E", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x4B, "Accelerator pedal position F", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x4C, "Commanded throttle actuator", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x4D, "Time with MIL on", "Minutes", (0.0, 65535.0), &|a, b, _, _|{ (a*256.0) + b });
        res.add_func_num(0x4E, "Time since trouble code cleared", "Minutes", (0.0, 65535.0), &|a, b, _, _|{ (a*256.0) + b });       
        res.add_func_str(0x51, "Fuel type", &|a, _, _, _| { PidList::get_fuel_type(a) });
        res.add_func_num(0x52, "Ethanol fuel percentage", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x53, "Absolute Evap system vapor pressure", "kPa", (0.0, 327.675), &|a, b, _, _|{ ((256.0 * a) + b) / 200.0 });
        res.add_func_num(0x54, "Evap system vapor pressure", "Pa", (-32767.0, 32768.0), &|a, b, _, _|{ ((256.0 * a) + b) - 32767.0 });

        res.add_func_mult(0x55, &["Short term secondary oxygen trim bank 1", "Short term secondary oxygen trim bank 3"], &["%", "%"], &[(-100.0, 99.2), (-100.0, 99.2)], &|a, b, _, _| { vec![(a * 1.28)-100.0, (b * 1.28)-100.0] });
        res.add_func_mult(0x56, &["Long term secondary oxygen trim bank 1", "Long term secondary oxygen trim bank 3"], &["%", "%"], &[(-100.0, 99.2), (-100.0, 99.2)], &|a, b, _, _| { vec![(a * 1.28)-100.0, (b * 1.28)-100.0] });
        res.add_func_mult(0x57, &["Short term secondary oxygen trim bank 2", "Short term secondary oxygen trim bank 4"], &["%", "%"], &[(-100.0, 99.2), (-100.0, 99.2)], &|a, b, _, _| { vec![(a * 1.28)-100.0, (b * 1.28)-100.0] });
        res.add_func_mult(0x58, &["Long term secondary oxygen trim bank 2", "Long term secondary oxygen trim bank 4"], &["%", "%"], &[(-100.0, 99.2), (-100.0, 99.2)], &|a, b, _, _| { vec![(a * 1.28)-100.0, (b * 1.28)-100.0] });
        res.add_func_num(0x59, "Fuel rail absolute pressure", "kPa", (0.0, 655350.0), &|a, b, _, _|{ 10.0*(a*256.0 + b) });
        res.add_func_num(0x5A, "Relative accelerator pedal position", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x5B, "Hybrid battery pack remaining life", "%", (0.0, 100.0), &|a, _, _, _|{ a / 2.55 });
        res.add_func_num(0x5C, "Engine oil temperature", "\u{00B0}C", (-40.0, 210.0), &|a, _, _, _|{ a - 40.0 });
        res.add_func_num(0x5D, "Fuel injection timing", "\u{00B0}", (0.0, 100.0), &|a, b, _, _|{ (((256.0 * a) + b) / 128.0) - 210.0 });
        res.add_func_num(0x5E, "Engine fuel rate", "L/h", (0.0, 3212.75), &|a, b, _, _|{ ((256.0 * a) + b) / 20.0 });

        res.add_func_num(0xC0, "Odometer reading", "km", (0.0, 429496729.5), &|a, b, c, d|{ ( (a * 2i32.pow(24) as f32)+(b * 2i32.pow(16) as f32)+(c * 2i32.pow(8) as f32) +d)/10.0 });
        res 
    }

    pub fn get_desc_pid(&self, pid: u8) -> Option<(u8, Vec<&'static str>)> {
        self.pids[pid as usize].as_ref().map(|x| (pid, x.desc.clone()))
    }

    
    fn add_func_num(&mut self, pid: u8, desc: &'static str, unit: &'static str, bounds: (f32, f32), f: &'static dyn Fn(f32,f32,f32,f32) -> f32) {
        self.pids[pid as usize] = Some(PidConvert {
            data_type: OBDDataType::Number,
            desc: vec![desc],
            unit: vec![unit],
            fun1: Arc::new(Box::new(f)),
            fun2: Arc::new(Box::new(|_, _, _, _| { vec![] })),
            fun3: Arc::new(Box::new(|_, _, _, _| "".into() )),
            bounds: vec![bounds],
        });
    }

    fn add_func_mult(&mut self, pid: u8, desc: &[&'static str], unit: &[&'static str], bounds: &[(f32, f32)], f: &'static dyn Fn(f32,f32,f32,f32) -> Vec<f32>) {
        assert!(desc.len() == unit.len() && unit.len() == bounds.len()); // TODO - Const generic when that becomes stable

        self.pids[pid as usize] = Some(PidConvert {
            data_type: OBDDataType::MultiNumber,
            desc: Vec::from(desc),
            unit: Vec::from(unit),
            fun1: Arc::new(Box::new(|_, _, _, _| {0.0f32 })),
            fun2: Arc::new(Box::new(f)),
            fun3: Arc::new(Box::new(|_, _, _, _| "".into())),
            bounds: Vec::from(bounds)

        })
    }

    fn add_func_str(&mut self, pid: u8, desc: &'static str, f: &'static dyn Fn(u8,u8,u8,u8) -> String) {
        self.pids[pid as usize] = Some(PidConvert {
            data_type: OBDDataType::String,
            desc: vec![desc],
            unit: vec![],
            fun1: Arc::new(Box::new(|_, _, _, _| {0.0f32 })),
            fun2: Arc::new(Box::new(|_, _, _, _| { vec![] })),
            fun3: Arc::new(Box::new(f)),
            bounds: vec![]
        })
    }

    pub fn parse_pid(&self, pid: u8, args: &[u8]) -> Option<PidReturnType> {
        let parser = self.pids[pid as usize].as_ref()?;
        let len = min(4, args.len());
        let mut n : [u8; 4] = [0x00; 4];
        n[0..len].copy_from_slice(&args[0..len]);
        Some(parser.parse(n))
    }

    fn get_fuel_status(a: u8) -> String {
        match a {
            0 => "Engine off",
            1 => "Open loop due to insufficient engine temperature",
            2 => "Closed loop, using O2 sensor feedback",
            4 => "Open loop due to engine load OR fuel cut due to deceleration",
            8 => "Open loop due to system failure",
            16 => "Closed loop, using at least 1 O2 sensor but fault detected",
            _ => return format!("Unknown. Raw: 0x{:02X}", a)
        }.into()
    }
    
    fn get_secondary_air_status(a: u8) -> String {
        match a {
            1 => "Upstream",
            2 => "Downstream of catalytic converter",
            4 => "From the outside atmosphere or off",
            8 => "Pump commanded on for diagnostics",
            _ => return format!("Unknown. Raw: 0x{:02X}", a)
        }.into()
    }

    fn get_obd_standard(a: u8) -> String {
        match a {
            1 => "OBD-II as defined by CARB",
            2 => "OBD as defined by EPA",
            3 => "OBD and OBD-II",
            4 => "OBD-I",
            5 => "Not OBD compliant",
            6 => "EOBD (Europe)",
            7 => "EOBD and OBD-II",
            8 => "EOBD and OBD",
            9 => "EOBD, OBD and OBD-II",
            10 => "JOBD (Japan)",
            11 => "JOBD and OBD-II",
            12 => "JOBD and EOBD",
            13 => "JOBD, EOBD and OBD-II",
            14 | 15 | 16 | 22 | 27 | 34..=250 => "Reserved",
            17 => "Engine manufacturer diagnostics (EMD)",
            18 => "Engine manufacturer diagnostics enhanced (EMB+)",
            19 => "Heavy Duty OBD (Child/Partial) (HD OBD-C)",
            20 => "Heavy Duty OBD (HD OBD)",
            23 => "heavy Duty Euro OBD Stage I without NOx control (HD EOBD-I)",
            24 => "heavy Duty Euro OBD Stage I with NOx control (HD EOBD-I N)",
            25 => "heavy Duty Euro OBD Stage II without NOx control (HD EOBD-II)",
            26 => "heavy Duty Euro OBD Stage II with NOx control (HD EOBD-II N)",
            28 => "Brazil OBD Phase 1 (OBDBr-1)",
            29 => "Brazil OBD Phase 2 (OBDBr-2)",
            30 => "Korean OBD (KOBD)",
            31 => "India OBD-I (IOBD-I)",
            32 => "India OBD-II (IOBD-II)",
            33 => "heavy Duty Euro OBD Stage IV (HD EOBD-IV)",
            251..=255 => "Not avaliable for assignment (SAE J1939 special meaning)",
            _ => return format!("Unknown. Raw: 0x{:02X}", a)
        }.into()
    }

    fn get_fuel_type(a: u8) -> String {
        match a {
            0 => "Not avaliable",
            1 => "Gasoline",
            2 => "Methanol",
            3 => "Ethanol",
            4 => "Diesel",
            5 => "LPG",
            6 => "CNG",
            7 => "Propane",
            8 => "Electric",
            9 => "Bifuel running Gasoline",
            10 => "Bifuel running Methanol",
            11 => "Bifuel running Ethanol",
            12 => "Bifuel running LPG",
            13 => "Bifuel running CNG",
            14 => "Bifuel running Propane",
            15 => "Bifuel running Electricity",
            16 => "Bifuel running electric and combustion engine",
            17 => "Hybrid Gasoline",
            18 => "Hybrid Ethanol",
            19 => "Hybrid Diesel",
            20 => "Hybrid Electric",
            21 => "Hybrid running electric and combustion engine",
            22 => "Hybrid Regenerative",
            23 => "Bifuel running Diesel",
            _ => return format!("Unknown. Raw: 0x{:02X}", a)
        }.into()
    }
}

#[derive(Debug, Clone)]
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
        println!("Supported PIDS:");
        for x in s01.get_supported_chartable_pids() {
            println!("0x{:02X?} {:?}", x.0, x.1);
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

    pub fn get_chartable_pid(&self, s: &ObdServer, pid: u8) -> OBDError<Option<PidReturnType>> {
        self.check_service_supported(pid)?;
        let bytes = s.run_command(0x01, &[pid])?;
        Ok(PID_LIST.parse_pid(pid, &bytes[2..]))
    }

    pub fn get_supported_chartable_pids(&self) -> Vec<(u8, Vec<&'static str>)> {
        (0x01..0xFF as u8)
            .filter(|x| self.check_service_supported(*x).is_ok())
            .map(|x| PID_LIST.get_desc_pid(x))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }
}