use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex, RwLock, atomic::AtomicBool, mpsc::{self, Receiver, Sender, channel}}, time::Instant};
use std::sync::atomic::Ordering::Relaxed;
use commapi::comm_api::{ComServer, ISO15765Config, ISO15765Data};

use crate::{commapi::{self, comm_api::ComServerError}, windows::diag_session::kwp2000_session::{self, KWP2000DiagSession}};

use super::{CautionLevel, CommandError, ECUCommand, DTC, ProtocolError, ProtocolResult, ProtocolServer, Selectable};

// Developed using Daimler's KWP2000 documentation
// http://read.pudn.com/downloads554/ebook/2284613/KWP2000_release2_2.pdf

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, )]
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

impl Service {
    pub fn to_vec() -> Vec<Service> {
        vec![
            //Self::StartDiagSession,
            Self::ECUReset,
            Self::ClearDiagnosticInformation,
            Self::ReadDTCStatus,
            Self::ReadDTCByStatus,
            Self::ReadECUID,
            Self::ReadDataByLocalID,
            Self::ReadDataByID,
            Self::ReadMemoryByAddress,
            Self::SecurityAccess,
            Self::DisableNormalMsgTransmission,
            Self::EnableNormalMsgTransmission,
            Self::DynamicallyDefineLocalID,
            Self::WriteDataByID,
            Self::IOCTLByLocalID,
            Self::StartRoutineByLocalID,
            Self::StopRoutineByLocalID,
            Self::RequestRoutineResultsByLocalID,
            Self::RequestDownload,
            Self::RequestUpload,
            Self::TransferData,
            Self::RequestTransferExit,
            Self::WriteDataByLocalID,
            Self::WriteMemoryByAddress,
            //Self::TesterPresent,
            Self::ControlDTCSettings,
            Self::ResponseOnEvent,
        ]
    }
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

    fn get_name(&self) -> String {
        format!("{:?}", &self)
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

impl ToString for Service {
    fn to_string(&self) -> String {
        format!("{}(0x{:02X})", self.get_name(), self.get_byte())
    }
}

impl ECUCommand for Service {
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

#[derive(Debug, Copy, Clone)]
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
    RoutineNotComplete,
    RequestOutOfRange, //0x31
    InvalidKey, // 0x35,
    ExceededAttempts, // 0x36,
    TimeDelayNotExpired, // 0x37,
    DownloadNotAccepted, // 0x40,
    UploadNotAccepted, // 0x50,
    TransferSuspended, // 0x71,
    DataDecompressionFailed, // 0x9A
    DataDecryptionFailed, // 0x9B,
    ECUNotResponding, // 0xA0,
    ECUAddressUnknown, //0xA1,
    SecurityAccessDenied,
    ResponsePending,
    ServiceNotSupportedActiveSession,
    CustomDaimler(u8), // DCX
    Reserved(u8),
    Unknown(u8)
}

impl CommandError for NegativeResponse {
    fn get_text(&self) -> String {
        match self {
            NegativeResponse::GeneralReject => "General reject",
            NegativeResponse::ServiceNotSupported => "Service is not supported",
            NegativeResponse::SubFunctionNotSupported => "Sub function not supported / invalid format",
            NegativeResponse::Busy => "ECU is currently busy performing another operation",
            NegativeResponse::RequestSequenceError => "Conditions are not correct or Request sequence error",
            NegativeResponse::RoutineNotComplete => "Routine is not yet completed",
            NegativeResponse::RequestOutOfRange => "The request is out of range",
            NegativeResponse::InvalidKey => "Invalid security key",
            NegativeResponse::ExceededAttempts => "Exceeded number of security access attempts",
            NegativeResponse::TimeDelayNotExpired => "The required time day has not yet expired",
            NegativeResponse::DownloadNotAccepted => "Download not accepted",
            NegativeResponse::UploadNotAccepted => "Upload not accepted",
            NegativeResponse::TransferSuspended => "Data transfer suspended",
            NegativeResponse::DataDecompressionFailed => "Data decompression failed",
            NegativeResponse::DataDecryptionFailed => "Data decryption failed",
            NegativeResponse::ECUNotResponding => "According to the gateway, the ECU is not responding",
            NegativeResponse::ECUAddressUnknown => "The gateway does not know what ECU address this is",
            NegativeResponse::SecurityAccessDenied => "Security access for this function was denied",
            NegativeResponse::ResponsePending => "Response pending...",
            NegativeResponse::ServiceNotSupportedActiveSession => "This services is not supported in the current diagnostic session",
            NegativeResponse::CustomDaimler(x) => return format!("Custom DaimlerChrysler DCX code 0x{:02X}", x),
            NegativeResponse::Reserved(x) => return format!("ISO 14230 Reserved code 0x{:02X}", x),
            NegativeResponse::Unknown(x) => return format!("Unknown error 0x{:02X}", x)
        }.into()
    }

    /// This displays a nice 'help message' for the user
    /// 
    ///
    /// 
    fn get_help(&self) -> Option<String> {
        match self {
            NegativeResponse::GeneralReject => None,
            NegativeResponse::ServiceNotSupported => Some("This service is not supported by the ECU".into()),
            NegativeResponse::SubFunctionNotSupported => Some("The arguments provided in the command may not be correct".into()),
            NegativeResponse::Busy => Some("The ECU is currently performing another operation, please wait".into()),
            NegativeResponse::RequestSequenceError => Some("The ECU requires something to be ran prior to running this command".into()),
            NegativeResponse::RoutineNotComplete => Some("The diagnostic routine was not completed".into()),
            NegativeResponse::RequestOutOfRange => Some("The data entered exceeded the maximum value that the ECU can read or store".into()),
            NegativeResponse::InvalidKey => Some("The wrong seed-key was entered to gain a higher security clearance".into()),
            NegativeResponse::ExceededAttempts => Some("You have exceeded the number of attempts to gain a higher security clearance".into()),
            NegativeResponse::TimeDelayNotExpired => Some("You have entered a seed-key response too quickly. Please wait.".into()),
            NegativeResponse::DownloadNotAccepted => None,
            NegativeResponse::UploadNotAccepted => None,
            NegativeResponse::TransferSuspended => Some("The data transfer was suspended due to an unknown fault".into()),
            NegativeResponse::DataDecompressionFailed => None,
            NegativeResponse::DataDecryptionFailed => None,
            NegativeResponse::ECUNotResponding => Some("In your car, the gateway talks to the ECU directly and has detected that the ECU has stopped responding".into()),
            NegativeResponse::ECUAddressUnknown => Some("In your car, the gateway is trying to talk to the ECU you requested, but you entered an unknown address".into()),
            NegativeResponse::SecurityAccessDenied => Some("In order to execute this function, you need to obtain a higher security clearance.".into()),
            NegativeResponse::ResponsePending => Some("The ECU is currently trying to send a response".into()),
            NegativeResponse::ServiceNotSupportedActiveSession => Some("This function is not supported in the current diagnostic session. Try to switch diagnostic sessions".into()),
            NegativeResponse::CustomDaimler(x) => Some("This error code is reserved by DaimlerChrysler. Therefore its meaning is unknown".into()),
            NegativeResponse::Reserved(x) => None,
            NegativeResponse::Unknown(x) => None,
        }
    }


    fn from_byte(b: u8) -> Self {
        match b {
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupported,
            0x21 => Self::Busy,
            0x22 => Self::RequestSequenceError,
            0x23 => Self::RoutineNotComplete, // As off 2002 this is deprecated
            0x31 => Self::RequestOutOfRange,
            0x33 => Self::SecurityAccessDenied,
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceededAttempts,
            0x37 => Self::TimeDelayNotExpired,
            0x40 => Self::DownloadNotAccepted,
            0x50 => Self::UploadNotAccepted,
            0x71 => Self::TransferSuspended,
            0x78 => Self::ResponsePending,
            0x80 => Self::ServiceNotSupportedActiveSession,
            0x9A => Self::DataDecompressionFailed,
            0x9B => Self::DataDecryptionFailed,
            0xA0 => Self::ECUNotResponding,
            0xA1 => Self::ECUAddressUnknown,
            (0x81..=0x8F) => Self::Reserved(b),
            (0x90..=0x99) => Self::CustomDaimler(b),
            (0xA2..=0xF9) => Self::CustomDaimler(b),
            0xFF => Self::Reserved(0xFF),
            _ => Self::Unknown(b)
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
    iso_tp_settings: ISO15765Config,
    should_run: Arc<AtomicBool>,
    last_error: Arc<RwLock<Option<ProtocolError>>>,
    cmd_tx: Sender<(u8, Vec<u8>, bool)>,
    cmd_rx: Arc<Receiver<ProtocolResult<Vec<u8>>>>,
    curr_session_type: DiagSession,
    send_id: u32,
    cmd_mutex: Arc<Mutex<()>>
}

#[derive(Debug, Clone)]
pub struct ECUIdentification {
    part_num: String,
    hw_version: String,
    sw_version: String,
    is_boot_sw: bool,
    variant: u32,

}

impl std::default::Default for ECUIdentification {
    fn default() -> Self {
        Self {
            part_num: "Unknown".into(),
            hw_version: "Unknown".into(),
            sw_version: "Unknown".into(),
            is_boot_sw: false,
            variant: 0,
        }
    }
}

fn bcd_decode(input: u8) -> String {
    let low = input & 0x0F;
    let high = (input & 0xF0) >> 4;
    return format!("{}{}", low, high);
}

impl KWP2000ECU {
    pub (crate) fn send_receive_kwp2000_cmd(server: &dyn ComServer, send_id: u32, cmd: u8, args: &[u8], receive_require: bool) -> std::result::Result<Vec<u8>, ProtocolError> {
        let mut data = ISO15765Data {
            id: send_id,
            data: vec![],
            pad_frame: false,
        };
        data.data.push(cmd);
        data.data.extend_from_slice(args);
        if !receive_require {
            server.send_iso15765_data(&[data], 0).map(|_| vec![]).map_err(ProtocolError::CommError)
        } else {
            // Await max 1 second for response
            let mut tmp_res = server.send_receive_iso15765(data, 1000, 1).map(|r| r[0].data.clone())?;
            if tmp_res[0] == 0x7F && tmp_res[2] == 0x78 { // ResponsePending
                println!("KWP2000 - ECU is processing request - Waiting!");
                let start = Instant::now();
                while start.elapsed().as_millis() < 500 {
                    // ECU is sending a response, but its busy right now. just gotta wait for the ECU to give us its response!
                    if let Some(msg) = server.read_iso15765_packets(0, 1)?.get(0) {
                        tmp_res = msg.data.clone();
                    }
                }
            }
            if tmp_res[0] == 0x7F {
                // Still error :(
                Err(ProtocolError::ProtocolError(Box::new(NegativeResponse::from_byte(tmp_res[2]))))
            } else if tmp_res[0] == (cmd + 0x40) {
                Ok(tmp_res)
            } else {
                eprintln!("KWP2000 - Command response did not match request? Send: {:02X} - Recv: {:02X}", cmd, tmp_res[0]);
                Err(ProtocolError::Timeout)
            }
        }
    }

    pub fn get_ecu_info_data(&self) -> std::result::Result<ECUIdentification, ProtocolError> {
        let mut res = self.run_command(Service::ReadECUID.get_byte(), &[0x87], 500)?;
        res.drain(..1);
        println!("RES: {:02X?}", res);
        let mut diag = ECUIdentification::default();
        let origin = res[2];
        let supplier_id = res[3];
        let variant = (res[4] as u32) << 8 | res[5] as u32;
        //let diag_version = res[5];
        let hw_major = bcd_decode(res[7]);
        let hw_minor = bcd_decode(res[8]);
        let sw_xx = bcd_decode(res[9]);
        let sw_yy = bcd_decode(res[10]);
        let sw_zz = bcd_decode(res[11]);
        let part_number = String::from_utf8(Vec::from(&res[11..])).unwrap();

        diag.part_num = part_number;
        diag.sw_version = format!("{} {} {}", sw_xx, sw_yy, sw_zz);
        diag.hw_version = format!("{}.{}", hw_major, hw_minor);
        diag.variant = variant;
        Ok(diag)
    }

    pub fn clear_errors(&self) -> std::result::Result<(), ProtocolError> {
        self.run_command(Service::ClearDiagnosticInformation.get_byte(), &[0xFF, 0x00], 1000)?;
        Ok(())
    }

    fn set_diag_session_mode(&mut self, mode: DiagSession) -> std::result::Result<(), ProtocolError> {
        match self.run_command(Service::StartDiagSession.get_byte(), &[mode as u8], 1000) {
            Ok(_) => {
                self.curr_session_type = mode; // Switch diagnostic modes!
                Ok(())
            },
            Err(e) => {
                self.curr_session_type = DiagSession::Normal; // Assume normal if something happens
                Err(e)
            }
        }
    }
}



impl ProtocolServer for KWP2000ECU {
    type Command = Service;

    fn start_diag_session(mut comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> ProtocolResult<Self> {
        comm_server.open_iso15765_interface(500_000, false).map_err(ProtocolError::CommError)?;
        comm_server.configure_iso15765(cfg).map_err(ProtocolError::CommError)?;
    

        let should_run = Arc::new(AtomicBool::new(true));
        let should_run_t = should_run.clone();

        let error = Arc::new(RwLock::new(None));
        let mut error_t = error.clone();

        let (channel_tx_sender, channel_tx_receiver): (Sender<(u8, Vec<u8>, bool)>, Receiver<(u8, Vec<u8>, bool)>) = mpsc::channel();
        let (channel_rx_sender, channel_rx_receiver): (Sender<ProtocolResult<Vec<u8>>>, Receiver<ProtocolResult<Vec<u8>>>) = mpsc::channel();

        // Enter extended diagnostic session (Full features)
        let s_id = cfg.send_id;
        std::thread::spawn(move || {
            println!("KWP2000 Diag server start!");
            let mut timer = Instant::now();
            while should_run_t.load(Relaxed) {
                if let Ok(data) = channel_tx_receiver.try_recv() {
                    let res = KWP2000ECU::send_receive_kwp2000_cmd(comm_server.as_ref(), s_id, data.0, &data.1, data.2);
                    channel_rx_sender.send(res);
                }
                if timer.elapsed().as_millis() >= 2000 {
                    if let Ok(res) = KWP2000ECU::send_receive_kwp2000_cmd(comm_server.as_ref(), s_id, Service::TesterPresent.get_byte(), &[0x01], true) {
                        println!("Tester present resp: {:02X?}", res);
                    }
                    timer = Instant::now();
                }
                std::thread::sleep(std::time::Duration::from_micros(100))
            }
            println!("KWP2000 Diag server stop!");
            comm_server.close_iso15765_interface();
        });

        let mut ecu = KWP2000ECU {
            iso_tp_settings: *cfg,
            should_run,
            last_error: error,
            cmd_tx: channel_tx_sender,
            cmd_rx: Arc::new(channel_rx_receiver),
            send_id: cfg.send_id,
            curr_session_type: DiagSession::Normal, // Assumed,
            cmd_mutex: Arc::new(Mutex::new(()))
        };

        if let Err(e) = ecu.set_diag_session_mode(DiagSession::ExtendedDiag) {
            ecu.should_run.store(false, Relaxed);
            return Err(e)
        }
        Ok(ecu)
    }

    fn exit_diag_session(&mut self) {
        self.should_run.store(false, Relaxed);
    }

    fn run_command(&self, cmd: u8, args: &[u8], max_timeout_ms: u128) -> ProtocolResult<Vec<u8>> {
        self.cmd_mutex.lock().unwrap(); // We are allowed to send / receive!
        self.cmd_tx.send((cmd, Vec::from(args), true));
        
        let resp = self.cmd_rx.recv().unwrap()?;
        if resp[0] == 0x7F {
            let neg_code = NegativeResponse::from_byte(resp[2]);
            Err(ProtocolError::ProtocolError(Box::new(neg_code)))
        } else {
            Ok(resp)
        }
    }

    fn read_errors(&self) -> ProtocolResult<Vec<DTC>> {
        // 0x02 - Request Hex DTCs as 2 bytes
        // 0xFF00 - Request all DTCs (Mandatory per KWP2000)
        let mut bytes = self.run_command(Service::ReadDTCByStatus.get_byte(), &[0x02, 0xFF, 0x00], 500)?;
        bytes.drain(..1);
        println!("{:02X?}", bytes);
        let count = bytes[0] as usize;
        bytes.drain(0..1);

        let mut res: Vec<DTC> = Vec::new();
        for _ in 0..count {
            let name = format!("{:02X}{:02X}", bytes[0], bytes[1]);
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

    fn is_in_diag_session(&self) -> bool {
        self.should_run.load(Relaxed) // Diag server self-terminates upon ECU Session error
    }

    fn get_last_error(&self) -> Option<String> {
       match self.last_error.read().unwrap().as_ref() {
           Some(x) => Some(x.get_text()),
           None => None
       }
    }
}