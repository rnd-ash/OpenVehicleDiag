use crate::commapi::comm_api::{ComServerError, ComServer, ISO15765Data};
use crate::windows::window::WindowStateName::ODBTools;
use crate::commapi::protocols::odb2::ODBProcessError::NoResponse;
use crate::commapi::protocols::vin::Vin;
use crate::commapi::protocols::odb2::ODBStandard::ODB1;

pub type Result<T> = std::result::Result<T, ODBProcessError>;

fn read_write_payload_isotp(server: &mut Box<dyn ComServer>, payload: &ODBRequest) -> Result<Vec<u8>> {
    server.open_iso15765_interface(500_000, false).map_err(|e| ODBProcessError::CommError(e))?;
    // Guess to use something appropriate for block size and sep time
    let send_data = ISO15765Data {
        id: 0x07DF, // Global request ID for ODB-II over CAN
        data: Vec::from(payload.to_vec()),
        pad_frame: false
    };

    let mut ret: Option<ISO15765Data> = None;
    let mut err: Option<ODBProcessError> = None;
    match server.add_iso15765_filter(0x07E8, 0xFFFF, 0x07DF) {
        Ok(f_id) => {
            // Good enough
            server.set_iso15765_params(10, 8);

            match server.send_iso15765_data(&[send_data], 0).map_err(|e| ODBProcessError::CommError(e)) {
                Ok(_) => {
                    // Send was OK, start polling
                    let start = std::time::Instant::now();
                    let mut found = false;

                    while start.elapsed().as_millis() < 250 && !found {
                        for msg in server.read_iso15765_packets(0, 10).unwrap_or(Vec::new()) {
                            if msg.data.len() > 0 {
                                found = true;
                                ret = Some(msg);
                                break;
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5))
                    }

                },
                Err(e) => err = Some(e)
            }

        },
        Err(e) => err = Some(ODBProcessError::CommError(e))
    };
    server.close_iso15765_interface();
    if let Some(e) = err {
        return Err(e)
    }
    return if let Some(data) = ret {
        return Ok(data.data)
    } else {
        Err(ODBProcessError::NoResponse)
    }
}

fn read_write_payload(server: &mut Box<dyn ComServer>, use_can: bool, payload: &ODBRequest) -> Result<ODBResponse> {
    let resp = match use_can {
        true => read_write_payload_isotp(server, payload),
        false => unimplemented!()
    };

    return if let Ok(p) = resp {
        if p.len() > 1 {
            if p[0] == payload.service | 0x40 {
                match payload.pid {
                    None => {
                        Ok(ODBResponse {
                            service: payload.service,
                            pid: None,
                            data: Vec::from(&p[1..]),
                        })
                    }
                    Some(pid) => {
                        if p[1] == pid {
                            Ok(ODBResponse {
                                service: payload.service,
                                pid: Some(pid),
                                data: Vec::from(&p[2..]),
                            })
                        } else {
                            Err(ODBProcessError::InvalidResponse("Response pid did not match request pid".into()))
                        }
                    }
                }
            } else {
                Err(ODBProcessError::InvalidResponse("Response service did not match request service".into()))
            }
        } else {
            Err(ODBProcessError::InvalidResponse("ECU Did not reply with enough data".into()))
        }
    } else {
        Err(resp.err().unwrap())
    }
}

#[derive(Clone, Debug)]
pub struct ODBRequest {
    service: u8,
    pid: Option<u8>,
    args: Vec<u8>
}

impl ODBRequest {
    fn new(service: u8, pid: u8) -> Self {
        return Self {
            service,
            pid: Some(pid),
            args: vec![]
        }
    }

    fn new_nopid(service: u8) -> Self {
        return Self {
            service,
            pid: None,
            args: vec![]
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        match self.pid {
            None => vec![self.service],
            Some(p) => {
                let mut ret = vec![self.service, p];
                ret.extend_from_slice(&self.args);
                ret
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ODBResponse {
    service: u8,
    pid: Option<u8>,
    data: Vec<u8>
}

#[derive(Clone, Debug)]
pub enum ODBProcessError {
    NoResponse,
    CommError(ComServerError),
    ServiceNotSupported,
    PIDNotSupported,
    InvalidResponse(String)
}

#[derive(Copy, Clone, Debug)]
pub struct Service01 {
    supported_pids: [bool; 0xFF]
}

impl Service01 {
    pub fn get_supported_pids(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        for (pos, b) in self.supported_pids.iter().enumerate() {
            if b == &true {
                res.push(pos as u8);
            }
        }
        return res;
    }

    fn write_to_supported(&mut self, data: Vec<u8>, start_id: usize) {
        println!("{:?}", data);
        let mut curr_idx = start_id;
        for i in 0..4 as usize {
            let curr_byte = data[i];
            for shift in 0..=7 as usize {
                self.supported_pids[curr_idx] = (curr_byte >> 7-shift) & 0x01 > 0;
                curr_idx += 1;
            }
        }
    }

    fn read_pid_supported(&self, server: &mut Box<dyn ComServer>, use_can: bool, pid: usize) -> Result<Vec<u8>> {
        if self.supported_pids[pid] {
            read_write_payload(server, use_can, &ODBRequest::new(0x01, pid as u8)).map(|r| r.data)
        } else {
            Err(ODBProcessError::PIDNotSupported)
        }
    }

    pub(crate) fn init(server: &mut Box<dyn ComServer>, use_can: bool) -> Result<Self> {
        let mut s01 = Service01 { supported_pids: [false; 0xFF] };
        s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0x00)).map(|r| r.data)?, 0x01);

        // Ask for the next round of supported PIDs
        if s01.supported_pids[0x20] {
            s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0x20)).map(|r| r.data)?, 0x21);
        }
        if s01.supported_pids[0x40] {
            s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0x40)).map(|r| r.data)?, 0x41);
        }
        if s01.supported_pids[0x60] {
            s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0x60)).map(|r| r.data)?, 0x61);
        }
        if s01.supported_pids[0x80] {
            s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0x80)).map(|r| r.data)?, 0x81);
        }
        if s01.supported_pids[0xA0] {
            s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0xA0)).map(|r| r.data)?, 0xA1);
        }
        if s01.supported_pids[0xC0] {
            s01.write_to_supported(read_write_payload(server, use_can, &ODBRequest::new(0x01, 0xC0)).map(|r| r.data)?, 0xC1);
        }
        Ok(s01)
    }

    fn a_b(src: Vec<u8>) -> (f32, f32) {
        (src[0] as f32, src[1] as f32)
    }

    fn a_b_d_c(src: Vec<u8>) -> (f32, f32, f32, f32) {
        (src[0] as f32, src[1] as f32, src[2] as f32, src[3] as f32)
    }

    /// Returns the calculated engine load in a range of 0-100%
    pub fn get_engine_load(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok((self.read_pid_supported(server, use_can, 0x04)?[0] as f32 / 2.55) as u32)
    }

    /// Returns the engine collant tempurature in range 0-40C
    pub fn get_engine_coolant_temp(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok(self.read_pid_supported(server, use_can, 0x05)?[0] as u32 - 40)
    }

    /// Returns short term fuel trim in bank 01
    /// Range -100% to 99.2%
    /// -100 - Reduce fuel (Too rich)
    /// 99.2 - Add fuel (Too lean)
    pub fn get_short_term_ft_b1(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
       Ok((self.read_pid_supported(server, use_can, 0x06)?[0] as f32 / 1.28) as u32 - 100)
    }

    /// Returns long term fuel trim in bank 01
    /// Range -100% to 99.2%
    /// -100 - Reduce fuel (Too rich)
    /// 99.2 - Add fuel (Too lean)
    pub fn get_long_term_ft_b1(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok((self.read_pid_supported(server, use_can, 0x07)?[0] as f32 / 1.28) as u32 - 100)
    }

    /// Returns short term fuel trim in bank 02
    /// Range -100% to 99.2%
    /// -100 - Reduce fuel (Too rich)
    /// 99.2 - Add fuel (Too lean)
    pub fn get_short_term_ft_b2(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok((self.read_pid_supported(server, use_can, 0x08)?[0] as f32 / 1.28) as u32 - 100)
    }

    /// Returns long term fuel trim in bank 02
    /// Range -100% to 99.2%
    /// -100 - Reduce fuel (Too rich)
    /// 99.2 - Add fuel (Too lean)
    pub fn get_long_term_ft_b2(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok((self.read_pid_supported(server, use_can, 0x09)?[0] as f32 / 1.28) as u32 - 100)
    }

    /// Returns the fuel pressure in kPa
    pub fn get_fuel_pressure(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok(self.read_pid_supported(server, use_can, 0x0A)?[0] as u32 * 3)
    }

    /// Returns the absolute pressure of the intake manifold in kPa
    pub fn get_intake_absolute_pressure(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok(self.read_pid_supported(server, use_can, 0x0B)?[0] as u32)
    }

    /// Returns the engine speed in RPM
    pub fn get_engine_rpm(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        let r = Service01::a_b(self.read_pid_supported(server, use_can, 0x0C)?);
        Ok((r.0*256.0 + r.1) as u32 / 4)
    }

    /// Returns the engine speed in RPM
    pub fn get_vehicle_speed(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok(self.read_pid_supported(server, use_can, 0x0D)?[0] as u32)
    }

    /// Returns the timing advance of the engine before Top dead center (TDC) in degrees.
    pub fn get_timing_advance(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok(self.read_pid_supported(server, use_can, 0x0E)?[0] as u32 / 2 - 64)
    }

    /// Returns the intake air temperature in degrees C
    pub fn get_intake_air_temp(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok(self.read_pid_supported(server, use_can, 0x0F)?[0] as u32  - 40)
    }

    /// Returns Mass airflow rate (MAF) in grams/sec
    pub fn get_maf_rate(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        let (a, b) =  Self::a_b(self.read_pid_supported(server, use_can, 0x10)?);
        Ok((256*a + b) / 100)
    }

    /// Returns throttle position in %
    pub fn get_throttle_pos(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<u32> {
        Ok((self.read_pid_supported(server, use_can, 0x11)?[0] as f32 * 100.0/255.0) as u32)
    }

    /// Returns Commanded secondary air status
    pub fn get_secondary_air_status(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<SecondaryAirStatus> {
        match self.read_pid_supported(server, use_can, 0x12)?[0] {
            0x01 => Ok(SecondaryAirStatus::UpStream),
            0x02 => Ok(SecondaryAirStatus::Downstream),
            0x04 => Ok(SecondaryAirStatus::Outside),
            0x08 => Ok(SecondaryAirStatus::PumpControlled),
            _ => Err(ODBProcessError::InvalidResponse("Secondary air status byte not valid".into()))
        }
    }

    /// Returns the ODB-II Standard the vehicle supports
    pub fn get_odb_std(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<Vec<ODBStandard>> {
        match self.read_pid_supported(server, use_can, 0x12)?[0] {
            01 => Ok(vec![ODBStandard::CARB]),
            02 => Ok(vec![ODBStandard::EPA]),
            03 => Ok(vec![ODBStandard::ODB1, ODBStandard::ODB2]),
            04 => Ok(vec![ODBStandard::ODB1]),
            05 => Ok(vec![ODBStandard::NA]),
            06 => Ok(vec![ODBStandard::EOBD]),
            07 => Ok(vec![ODBStandard::EOBD, ODBStandard::ODB2]),
            08 => Ok(vec![ODBStandard::EOBD, ODBStandard::ODB1]),
            09 => Ok(vec![ODBStandard::EOBD, ODBStandard::ODB1, ODBStandard::ODB2]),
            10 => Ok(vec![ODBStandard::JOBD]),
            11 => Ok(vec![ODBStandard::JOBD, ODBStandard::ODB2]),
            12 => Ok(vec![ODBStandard::JOBD, ODBStandard::EOBD]),
            13 => Ok(vec![ODBStandard::JOBD, ODBStandard::EOBD, ODBStandard::ODB2]),
            14 | 15 | 16 => Ok(vec![ODBStandard::Reserved]),
            17 => Ok(vec![ODBStandard::EMD]),
            18 => Ok(vec![ODBStandard::EMD_Plus]),
            19 => Ok(vec![ODBStandard::HD_OBD_C]),
            20 => Ok(vec![ODBStandard::HD_OBD]),
            21 => Ok(vec![ODBStandard::WWH_OBD]),
            22 => Ok(vec![ODBStandard::Reserved]),
            23 => Ok(vec![ODBStandard::HD_EOBD1]),
            24 => Ok(vec![ODBStandard::HD_EOBD1_N]),
            25 => Ok(vec![ODBStandard::HD_EOBD2]),
            26 => Ok(vec![ODBStandard::HD_EOBD2_N]),
            27 => Ok(vec![ODBStandard::Reserved]),
            28 => Ok(vec![ODBStandard::ODB_BR_1]),
            29 => Ok(vec![ODBStandard::ODB_BR_2]),
            30 => Ok(vec![ODBStandard::KOBD]),
            31 => Ok(vec![ODBStandard::IODB1]),
            32 => Ok(vec![ODBStandard::IODB2]),
            33 => Ok(vec![ODBStandard::HD_EOBD_5]),
            34..=250 => Ok(vec![ODBStandard::Reserved]),
            _ => Err(ODBProcessError::InvalidResponse("ODB Standard is SAE J1939 special".into()))
        }
    }

    /// Returns the fuel type the engine is running
    pub fn get_engine_fuel_type(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<FuelType> {
        match self.read_pid_supported(server, use_can, 0x51)?[0] {
            0 => Ok(FuelType::NA),
            1 => Ok(FuelType::Gasoline),
            2 => Ok(FuelType::Methanol),
            3 => Ok(FuelType::Ethanol),
            4 => Ok(FuelType::Diesel),
            5 => Ok(FuelType::LPG),
            6 => Ok(FuelType::CNG),
            7 => Ok(FuelType::Propane),
            8 => Ok(FuelType::Electric),
            9 => Ok(FuelType::BifuelGasoline),
            10 => Ok(FuelType::BifuelMethanol),
            11 => Ok(FuelType::BifuelEthanol),
            12 => Ok(FuelType::BifuelLPG),
            13 => Ok(FuelType::BifuelCNG),
            14 => Ok(FuelType::BifuelPropane),
            15 => Ok(FuelType::BifuelElectricity),
            16 => Ok(FuelType::BifuelElectricCombustion),
            17 => Ok(FuelType::HybridGasoline),
            18 => Ok(FuelType::HybridEthanol),
            19 => Ok(FuelType::HybridDiesel),
            20 => Ok(FuelType::HybridElectric),
            21 => Ok(FuelType::HybridElectricCombustion),
            22 => Ok(FuelType::HybridRegen),
            23 => Ok(FuelType::Diesel),
            _ => Err(ODBProcessError::InvalidResponse("Fuel type is invalid".into()))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SecondaryAirStatus {
    /// Upstream
    UpStream,
    /// Downstream from catalytic converter
    Downstream,
    /// From the outside atmosphere or off
    Outside,
    /// Pump commanded on for diagnostics
    PumpControlled
}


#[derive(Debug, Copy, Clone)]
pub enum ODBStandard {
    CARB,
    EPA,
    ODB2,
    ODB1,
    NA,
    EOBD,
    JOBD,
    Reserved,
    EMD,
    EMD_Plus,
    HD_OBD_C,
    HD_OBD,
    WWH_OBD,
    HD_EOBD1,
    HD_EOBD1_N,
    HD_EOBD2,
    HD_EOBD2_N,
    ODB_BR_1,
    ODB_BR_2,
    KOBD,
    IODB1,
    IODB2,
    HD_EOBD_5,
}

pub enum FuelType {
    NA,
    Gasoline,
    Methanol,
    Ethanol,
    Diesel,
    LPG,
    CNG,
    Propane,
    Electric,
    BifuelGasoline,
    BifuelMethanol,
    BifuelEthanol,
    BifuelLPG,
    BifuelCNG,
    BifuelPropane,
    BifuelElectricity,
    BifuelElectricCombustion,
    HybridGasoline,
    HybridEthanol,
    HybridDiesel,
    HybridElectric,
    HybridElectricCombustion,
    HybridRegen,
    BifuelDiesel
}



#[derive(Copy, Clone, Debug)]
pub struct Service03;

impl Service03 {
    pub fn get_error_codes(server: &mut Box<dyn ComServer>, use_can: bool) -> Result<()> {
        read_write_payload(server, use_can, &ODBRequest::new_nopid(0x03)).map(|mut res| {
            println!("{:02X?}", res)
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Service09 {
    VINMessageCount: bool,
    VIN: bool,
    CalibrationIDCount: bool,
    CalibrationID: bool,
    CalibrationVerificationCount: bool,
    CalibrationVerification: bool,
    PerfTrackingCount: bool,
    PerfTracking: bool,
    ECUNameCount: bool,
    ECUName: bool,
    CompressionPerfTest: bool
}
impl Service09 {
    pub fn get_vin(&self, server: &mut Box<dyn ComServer>, use_can: bool) -> Result<Vin> {
        if self.VIN == false {
            return Err(ODBProcessError::PIDNotSupported)
        }
        let mut data = read_write_payload(server, use_can, &ODBRequest::new(0x09, 0x02))?.data;
        data.drain(0..1);
        return if let Some(x) = Vin::new(String::from_utf8(data).unwrap()) {
            Ok(x)
        } else {
            Err(ODBProcessError::InvalidResponse("VIN is not correct length".into()))
        }
    }

    pub fn init(server: &mut Box<dyn ComServer>, use_can: bool) -> Result<Self> {
        read_write_payload(server, use_can, &ODBRequest::new(0x09, 0x00)).map(|mut res| {
            Service09 {
                VINMessageCount: res.data[0] >> 7 & 0x01 > 0,
                VIN: res.data[0] >> 6 & 0x01 > 0,
                CalibrationIDCount: res.data[0] >> 5 & 0x01 > 0,
                CalibrationID: res.data[0] >> 4 & 0x01 > 0,
                CalibrationVerificationCount: res.data[0] >> 3 & 0x01 > 0,
                CalibrationVerification: res.data[0] >> 2 & 0x01 > 0,
                PerfTrackingCount: res.data[0] >> 1 & 0x01 > 0,
                PerfTracking: res.data[0] >> 0 & 0x01 > 0,
                ECUNameCount: res.data[1] >> 7 & 0x01 > 0,
                ECUName: res.data[1] >> 6 & 0x01 > 0,
                CompressionPerfTest: res.data[1] >> 5 & 0x01 > 0,
            }
        })
    }
}