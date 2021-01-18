use std::{sync::{Arc, atomic::AtomicBool}, todo};
use std::sync::atomic::Ordering::Relaxed;
use commapi::comm_api::{ComServer, ISO15765Config, ISO15765Data};

use crate::commapi::{self, comm_api::ComServerError};

use super::{CautionLevel, CommandError, CommandLevel, DTC, ProtocolError, ProtocolResult, ProtocolServer, Selectable};

// Developed using Daimler's KWP2000 documentation
// http://read.pudn.com/downloads554/ebook/2284613/KWP2000_release2_2.pdf

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Service {
    StartDiagSession,
    ECUReset,
    ClearDiagnosticInformation,
    ReadDTCStatus,
    ReadDTCByStatus,
    ReadECUID,
    ReadDataByLocalID,
    ReadDataByID,
    ReadMemoryByAddress,
    SecurityAccess,
    DisableNormalMsgTransmission,
    EnableNormalMsgTransmission,
    DynamicallyDefineLocalID,
    WriteDataByID,
    IOCTLByLocalID,
    StartRoutineByLocalID,
    StopRoutineByLocalID,
    RequestRoutineResultsByLocalID,
    RequestDownload,
    RequestUpload,
    TransferData,
    RequestTransferExit,
    WriteDataByLocalID,
    WriteMemoryByAddress,
    TesterPresent,
    ControlDTCSettings,
    ResponseOnEvent,
    SupplierCustom(u8),
}

impl Selectable for Service {
    fn get_byte(&self) -> u8 {
        match self {
            Service::StartDiagSession => 0x10,
            Service::ECUReset => 0x11,
            Service::ClearDiagnosticInformation => 0x14,
            Service::ReadDTCStatus => 0x17,
            Service::ReadDTCByStatus => 0x18,
            Service::ReadECUID => 0x1A,
            Service::ReadDataByLocalID => 0x21,
            Service::ReadDataByID => 0x22,
            Service::ReadMemoryByAddress => 0x23,
            Service::SecurityAccess => 0x27,
            Service::DisableNormalMsgTransmission => 0x28,
            Service::EnableNormalMsgTransmission => 0x29,
            Service::DynamicallyDefineLocalID => 0x2C,
            Service::WriteDataByID => 0x2E,
            Service::IOCTLByLocalID => 0x30,
            Service::StartRoutineByLocalID => 0x31,
            Service::StopRoutineByLocalID => 0x32,
            Service::RequestRoutineResultsByLocalID => 0x33,
            Service::RequestDownload => 0x34,
            Service::RequestUpload => 0x35,
            Service::TransferData => 0x36,
            Service::RequestTransferExit => 0x37,
            Service::WriteDataByLocalID => 0x3B,
            Service::WriteMemoryByAddress => 0x3D,
            Service::TesterPresent => 0x3E,
            Service::ControlDTCSettings => 0x85,
            Service::ResponseOnEvent => 0x86,
            Service::SupplierCustom(sid) => *sid
        }
    }

    fn get_desc(&self) -> String {
        match self {
            Service::StartDiagSession => "Start diagnostic session",
            Service::ECUReset => "Reset ECU",
            Service::ClearDiagnosticInformation => "Clear diagnostic information",
            Service::ReadDTCStatus => "Read diagnostic trouble status",
            Service::ReadDTCByStatus => "Read diagnostic trouble codes by status",
            Service::ReadECUID => "Read ECU Identification data",
            Service::ReadDataByLocalID => "Read data by local ID",
            Service::ReadDataByID => "Read data by ID",
            Service::ReadMemoryByAddress => "Read memory by address",
            Service::SecurityAccess => "Security access",
            Service::DisableNormalMsgTransmission => "Disable normal message transmission",
            Service::EnableNormalMsgTransmission => "Enable normal message transmission",
            Service::DynamicallyDefineLocalID => "Dynamically define local ID",
            Service::WriteDataByID => "Write data by ID",
            Service::IOCTLByLocalID => "IOCTL by local ID",
            Service::StartRoutineByLocalID => "Start routine by local ID",
            Service::StopRoutineByLocalID => "Stop routine by local ID",
            Service::RequestRoutineResultsByLocalID => "Request routine results by local ID",
            Service::RequestDownload => "Request download",
            Service::RequestUpload => "Request upload",
            Service::TransferData => "Transfer data",
            Service::RequestTransferExit => "Request transfer exit",
            Service::WriteDataByLocalID => "Write data by local ID",
            Service::WriteMemoryByAddress => "Write memory by address",
            Service::TesterPresent => "Tester present",
            Service::ControlDTCSettings => "Control DTC Settings",
            Service::ResponseOnEvent => "Response on event",
            Service::SupplierCustom(x) => return format!("Custom({:02X})", x)
        }.into()
    }
}

impl CommandLevel for Service {
    fn get_caution_level(&self) -> CautionLevel {
        match &self {
            Service::StartDiagSession => CautionLevel::None,
            Service::ECUReset => CautionLevel::Warn,
            Service::ClearDiagnosticInformation => CautionLevel::None,
            Service::ReadDTCStatus => CautionLevel::None,
            Service::ReadDTCByStatus => CautionLevel::None,
            Service::ReadECUID => CautionLevel::None,
            Service::ReadDataByLocalID => CautionLevel::Alert,
            Service::ReadDataByID => CautionLevel::Alert,
            Service::ReadMemoryByAddress => CautionLevel::Alert,
            Service::SecurityAccess => CautionLevel::Warn,
            Service::DisableNormalMsgTransmission => CautionLevel::Alert,
            Service::EnableNormalMsgTransmission => CautionLevel::Alert,
            Service::DynamicallyDefineLocalID => CautionLevel::Alert,
            Service::WriteDataByID => CautionLevel::Alert,
            Service::IOCTLByLocalID => CautionLevel::Alert,
            Service::StartRoutineByLocalID => CautionLevel::Alert,
            Service::StopRoutineByLocalID => CautionLevel::Alert,
            Service::RequestRoutineResultsByLocalID => CautionLevel::Alert,
            Service::RequestDownload => CautionLevel::Alert,
            Service::RequestUpload => CautionLevel::Alert,
            Service::TransferData => CautionLevel::Alert,
            Service::RequestTransferExit => CautionLevel::Alert,
            Service::WriteDataByLocalID => CautionLevel::Alert,
            Service::WriteMemoryByAddress => CautionLevel::Alert,
            Service::TesterPresent => CautionLevel::None,
            Service::ControlDTCSettings => CautionLevel::Warn,
            Service::ResponseOnEvent => CautionLevel::Warn,
            Service::SupplierCustom(_) => CautionLevel::Warn,
        }
    }
}

enum DiagSession {
    Normal = 0x81,
    ECUFlash = 0x85,
    StandBy = 0x89,
    ECUPassive = 0x90,
    ExtendedDiag = 0x92,
}

#[derive(Debug, Copy, Clone)]
pub enum NegativeResponse {
    GeneralReject,
    ServiceNotSupported,
    SubFunctionNotSupported,
    Busy,
    RequestSequenceError,
    SecurityAccessDenied,
    ResponsePending,
    ServiceNotSupportedActiveSession,
    Custom(u8)
}

impl CommandError for NegativeResponse {
    fn get_desc(&self) -> String {
        todo!()
    }

    fn get_name(&self) -> String {
        return format!("{:?}", &self);
    }

    fn get_byte(&self) -> u8 {
        todo!()
    }

    fn from_byte(b: u8) -> Self {
        match b {
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupported,
            0x21 => Self::Busy,
            0x22 => Self::RequestSequenceError,
            0x33 => Self::SecurityAccessDenied,
            0x78 => Self::ResponsePending,
            0x80 => Self::ServiceNotSupportedActiveSession,
            _ => Self::Custom(b)
        }
    }
}


enum ResetType {
    PowerOnReset = 0x01,
    // NonVolativeMemoryReset = 0x82 // Don't support this as its optional
}

enum ClearDTCType {
    AllDTCs = 0xFF00
}



#[derive(Debug, Clone)]
pub struct KWP2000ECU {
    comm_server: Box<dyn ComServer>,
    iso_tp_settings: ISO15765Config,
    should_run: Arc<AtomicBool>,
    stop_tester_present: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct ECUIdentification {
    part_num: String,
    hw_version: String,
    sw_version: String,
    is_boot_sw: bool,
    diag_version: u8,
    varient: u8,

}

impl std::default::Default for ECUIdentification {
    fn default() -> Self {
        Self {
            part_num: "Unknown".into(),
            hw_version: "Unknown".into(),
            sw_version: "Unknown".into(),
            is_boot_sw: false,
            diag_version: 0,
            varient: 0,
        }
    }
}

fn bcd_decode(input: u8) -> String {
    let low = input & 0x0F;
    let high = (input & 0xF0) >> 4;
    return format!("{}{}", low, high);
}

impl KWP2000ECU {
    pub (crate) fn send_kwp2000_cmd(server: &dyn ComServer, send_id: u32, cmd: Service, args: &[u8]) -> std::result::Result<usize, ComServerError> {
        let mut data = ISO15765Data {
            id: send_id,
            data: vec![],
            pad_frame: false,
        };
        data.data.push(cmd.get_byte());
        data.data.extend_from_slice(args);
        server.send_iso15765_data(&[data], 0)
    }

    pub fn get_ecu_info_data(&self) -> std::result::Result<ECUIdentification, ProtocolError> {
        let res = self.run_command(Service::ReadECUID, &[0x87], 500)?;
        let mut diag = ECUIdentification::default();
        let origin = res[2];
        let supplier_id = res[3];
        let varient = res[4] & (0b1111110);
        let diag_version = res[5];
        let hw_major = bcd_decode(res[7]);
        let hw_minor = bcd_decode(res[8]);
        let sw_xx = bcd_decode(res[9]);
        let sw_yy = bcd_decode(res[10]);
        let sw_zz = bcd_decode(res[11]);
        let part_number = String::from_utf8(Vec::from(&res[11..])).unwrap();

        diag.part_num = part_number;
        diag.sw_version = format!("{} {} {}", sw_xx, sw_yy, sw_zz);
        diag.hw_version = format!("{}.{}", hw_major, hw_minor);
        diag.is_boot_sw = diag_version > 0xDF;
        diag.diag_version = diag_version;
        diag.varient = varient;
        Ok(diag)
    }

    pub fn clear_errors(&self) -> std::result::Result<(), ProtocolError> {
        self.run_command(Service::ClearDiagnosticInformation, &[0xFF, 0x00], 1000)?;
        Ok(())
    }
}

impl ProtocolServer for KWP2000ECU {
    type Command = Service;
    fn start_diag_session(mut comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> std::result::Result<Self, ProtocolError> {
        comm_server.open_iso15765_interface(500_000, false).map_err(ProtocolError::CommError)?;
        comm_server.add_iso15765_filter(
            cfg.recv_id,
            0xFFF, 
            cfg.send_id
        ).map_err(ProtocolError::CommError)?;
        comm_server.set_iso15765_params(cfg.sep_time, cfg.block_size).map_err(ProtocolError::CommError)?;
    

        let should_run = Arc::new(AtomicBool::new(true));
        let stop_send_tester_present = Arc::new(AtomicBool::new(true));

        let should_run_t = should_run.clone();
        let stop_tester_present_t = stop_send_tester_present.clone();

        let server_t = comm_server.clone();
        let ecu_id = cfg.send_id;
        // Enter extended diagnostic session (Full features)
        std::thread::spawn(move || {
            let mut last_send = std::time::Instant::now();
            println!("DIAG SERVER START");
            while should_run_t.load(Relaxed) {
                if last_send.elapsed().as_millis() > 2500 {
                    last_send = std::time::Instant::now();
                    if !stop_tester_present_t.load(Relaxed) {
                        // Tell the ECU Tester is still here, no response required though :)
                        KWP2000ECU::send_kwp2000_cmd(server_t.as_ref(), ecu_id, Service::TesterPresent, &[0x01]);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            println!("DIAG SERVER STOP");
        });



        let mut ecu = KWP2000ECU {
            comm_server,
            iso_tp_settings: *cfg,
            stop_tester_present: stop_send_tester_present,
            should_run,

        };
        if let Err(e) = ecu.run_command(Service::StartDiagSession, &[0x92], 250) {
            eprintln!("Error sending tester present {:?}", e);
            ecu.comm_server.close_iso15765_interface();
            Err(e)
        } else {
            ecu.stop_tester_present.store(false, Relaxed);
            Ok(ecu)
        }
    
    }

    fn exit_diag_session(&mut self) {
        self.should_run.store(false, Relaxed);
        self.comm_server.close_iso15765_interface();
    }

    fn run_command(&self, cmd: Self::Command, args: &[u8], max_timeout_ms: u128) -> ProtocolResult<Vec<u8>> {
        if let Err(e) =  KWP2000ECU::send_kwp2000_cmd(self.comm_server.as_ref(), self.iso_tp_settings.send_id, cmd, args) {
            self.stop_tester_present.store(false, Relaxed); // Wait for response
            return Err(ProtocolError::CommError(e));
        }
        if max_timeout_ms == 0 {
            Ok(vec![])
        } else {
            let start = std::time::Instant::now();
            while start.elapsed().as_millis() < max_timeout_ms {
                if let Ok(msgs) = self.comm_server.read_iso15765_packets(0, 1) {
                    for m in msgs {
                        if !m.data.is_empty() { // Avoid FF indications
                            if m.data[0] == cmd.get_byte() + 0x40 {
                                self.stop_tester_present.store(false, Relaxed);
                                return Ok(Vec::from(&m.data[1..]))
                            } else if m.data[0] == 0x7F { // Negative response
                                // data[1] = SID
                                if m.data[2] == 0x78 { // Response pending!
                                    self.stop_tester_present.store(true, Relaxed); // STOP Sending Tester present!
                                } else {
                                    // Other error
                                    self.stop_tester_present.store(false, Relaxed);
                                    return Err(ProtocolError::ProtocolError(NegativeResponse::from_byte(m.data[1]).get_name()))
                                }
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            self.stop_tester_present.store(false, Relaxed);
            Err(ProtocolError::Timeout)
        }
    }


    fn read_errors(&self) -> std::result::Result<Vec<DTC>, ProtocolError> {
        // 0x02 - Request Hex DTCs as 2 bytes
        // 0xFF00 - Request all DTCs (Mandatory per KWP2000)
        let mut bytes = self.run_command(Service::ReadDTCByStatus, &[0x02, 0xFF, 0x00], 500)?;
        println!("{:02X?}", bytes);
        let count = bytes[0] as usize;
        bytes.drain(0..1);

        let mut res: Vec<DTC> = Vec::new();
        for _ in 0..count {
            let code = match bytes[0] {
                x if x < 0x40 => 'P', // Powertrain DTC
                x if x < 0x80 => 'B', // Body DTC
                x if x < 0xC0 => 'C', // Chassis DTC
                x if x < 0xFF => 'N', // Network DTC
                _ => '?', // WTF is this error??
            };
            let name = format!("{}{:02X}{:02X}", code, bytes[0], bytes[1]);
            let status = bytes[2];
            println!("{:08b}", status);
            let flag = (status >> 4 & 0b00000001) > 0;
            let storage_state = (status >> 6) & 0b0000011;
            let mil = (status >> 7 & 0b00000001) > 0;
            res.push(DTC {
                error: name,
                present: flag,
                stored: storage_state > 0,
                check_engine_on: mil,

            });
            bytes.drain(0..3);
        }
        Ok(res)
    }
}