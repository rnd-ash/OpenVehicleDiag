use std::{borrow::Borrow, cell::RefCell, rc::Rc, sync::{Arc, Mutex, RwLock, atomic::AtomicBool, mpsc::{self, Receiver, Sender, channel}}, time::Instant};
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
        self.get_name()
    }
}

impl Into<u8> for Service {
    fn into(self) -> u8 {
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
            Service::SupplierCustom(sid) => sid
        }
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DiagSession {
    Normal = 0x81,
    ECUFlash = 0x85,
    StandBy = 0x89,
    ECUPassive = 0x90,
    ExtendedDiag = 0x92,
}

impl DiagSession {
    pub (crate) fn send_tester_present(&self) -> bool {
        self != &DiagSession::Normal
    }
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
            NegativeResponse::CustomDaimler(_) => Some("This error code is reserved by DaimlerChrysler. Therefore its meaning is unknown".into()),
            NegativeResponse::Reserved(_) => None,
            NegativeResponse::Unknown(_) => None,
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

#[derive(Debug, Clone)]
pub struct KWP2000ECU {
    iso_tp_settings: ISO15765Config,
    should_run: Arc<AtomicBool>,
    last_error: Arc<RwLock<Option<ProtocolError>>>,
    cmd_tx: Sender<(u8, Vec<u8>, bool)>,
    cmd_rx: Arc<Receiver<ProtocolResult<Vec<u8>>>>,
    curr_session_type: Arc<RwLock<DiagSession>>,
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

    pub fn clear_errors(&self) -> std::result::Result<(), ProtocolError> {
        self.run_command(Service::ClearDiagnosticInformation.into(), &[0xFF, 0x00], 1000)?;
        Ok(())
    }

    fn set_diag_session_mode(&mut self, mode: DiagSession) -> std::result::Result<(), ProtocolError> {
        match self.run_command(Service::StartDiagSession.into(), &[mode as u8], 1000) {
            Ok(_) => {
                *self.curr_session_type.write().unwrap() = mode; // Switch diagnostic modes!
                Ok(())
            },
            Err(e) => {
                *self.curr_session_type.write().unwrap() = DiagSession::Normal; // Assume normal if something happens
                Err(e)
            }
        }
    }

    pub fn get_session_type(&self) -> DiagSession {
        *self.curr_session_type.read().unwrap()
    }
}



impl ProtocolServer for KWP2000ECU {
    type Command = Service;
    type Error = NegativeResponse;
    fn start_diag_session(mut comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> ProtocolResult<Self> {
        comm_server.open_iso15765_interface(500_000, false).map_err(ProtocolError::CommError)?;
        comm_server.configure_iso15765(cfg).map_err(ProtocolError::CommError)?;

        let should_run = Arc::new(AtomicBool::new(true));
        let should_run_t = should_run.clone();

        let last_error = Arc::new(RwLock::new(None));
        let last_error_t = last_error.clone();

        let (channel_tx_sender, channel_tx_receiver): (Sender<(u8, Vec<u8>, bool)>, Receiver<(u8, Vec<u8>, bool)>) = mpsc::channel();
        let (channel_rx_sender, channel_rx_receiver): (Sender<ProtocolResult<Vec<u8>>>, Receiver<ProtocolResult<Vec<u8>>>) = mpsc::channel();

        let session_type = Arc::new(RwLock::new(DiagSession::Normal));
        let session_type_t = session_type.clone();

        // Enter extended diagnostic session (Full features)
        let s_id = cfg.send_id;
        std::thread::spawn(move || {
            println!("Diag server start!");
            let mut timer = Instant::now();
            while should_run_t.load(Relaxed) {
                if let Ok(data) = channel_tx_receiver.try_recv() {
                    let res = Self::run_command_isotp(comm_server.as_ref(), s_id, data.0, &data.1, data.2);
                    if channel_rx_sender.send(res).is_err() {
                        *last_error_t.write().unwrap() = Some(ProtocolError::CustomError("Sender channel died".into()));
                        break
                    }
                }
                if timer.elapsed().as_millis() >= 2000 && session_type_t.read().unwrap().send_tester_present() {
                    if let Ok(res) = Self::run_command_isotp(comm_server.as_ref(), s_id, Service::TesterPresent.into(), &[0x01], true) {
                        println!("Tester present resp: {:02X?}", res);
                    }
                    timer = Instant::now();
                }
                std::thread::sleep(std::time::Duration::from_micros(100))
            }
            println!("Diag server stop!");
            comm_server.close_iso15765_interface();
        });

        let mut ecu = KWP2000ECU {
            iso_tp_settings: *cfg,
            should_run,
            last_error,
            cmd_tx: channel_tx_sender,
            cmd_rx: Arc::new(channel_rx_receiver),
            send_id: cfg.send_id,
            curr_session_type: session_type, // Assumed,
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

    fn run_command(&self, cmd: u8, args: &[u8], _max_timeout_ms: u128) -> ProtocolResult<Vec<u8>> {
        let _guard = self.cmd_mutex.lock().unwrap(); // We are allowed to send / receive!
        if self.cmd_tx.send((cmd, Vec::from(args), true)).is_err() {
            return Err(ProtocolError::CustomError("Channel Tx failed".into()))
        }
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
        let mut bytes = self.run_command(Service::ReadDTCByStatus.into(), &[0x02, 0xFF, 0x00], 500)?;
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
            bytes.drain(0..3); // DTC is 3 bytes (1 for status, 2 for the ID)
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