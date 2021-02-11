
use std::{sync::{Arc, Mutex, RwLock, atomic::AtomicBool, mpsc::{self, Receiver, Sender}}, time::Instant};
use std::sync::atomic::Ordering::Relaxed;
use crate::commapi::comm_api::{ComServer, ISO15765Config};
use self::diag_session_control::DiagSession;
use super::{CautionLevel, CommandError, DTC, ECUCommand, ProtocolError, ProtocolResult, ProtocolServer, Selectable};

pub mod diag_session_control;

#[derive(Copy, Clone, Debug, Eq, PartialOrd, PartialEq)]
/// UDS Commands AKA SID (Service identifiers)
pub enum UDSCommand {
    DiagnosticSessionControl,
    ECUReset,
    ClearDTCInformation,
    ReadDTCInformation,
    ReadDataByID,
    ReadMemoryByAddress,
    ReadScalingDataById,
    SecurityAccess,
    CommunicationControl,
    Authentication,
    ReadDataByPeriodicID,
    DynamicDefineDataId,
    WriteDataByID,
    IOCTLById,
    RoutineControl,
    RequestDownload,
    RequestUpload,
    TransferData,
    TransferExit,
    WriteMemoryByAddress,
    TesterPresent,
    RequestFileTransfer,
    ControlDTCSetting,
    LinkControl,
}

impl Selectable for UDSCommand {
    fn get_name(&self) -> String {
        format!("{:?}", &self)
    }

    fn get_desc(&self) -> String {
        match &self {
            UDSCommand::DiagnosticSessionControl => {}
            UDSCommand::ECUReset => {}
            UDSCommand::ClearDTCInformation => {}
            UDSCommand::ReadDTCInformation => {}
            UDSCommand::ReadDataByID => {}
            UDSCommand::ReadMemoryByAddress => {}
            UDSCommand::ReadScalingDataById => {}
            UDSCommand::SecurityAccess => {}
            UDSCommand::CommunicationControl => {}
            UDSCommand::Authentication => {}
            UDSCommand::ReadDataByPeriodicID => {}
            UDSCommand::DynamicDefineDataId => {}
            UDSCommand::WriteDataByID => {}
            UDSCommand::IOCTLById => {}
            UDSCommand::RoutineControl => {}
            UDSCommand::RequestDownload => {}
            UDSCommand::RequestUpload => {}
            UDSCommand::TransferData => {}
            UDSCommand::TransferExit => {}
            UDSCommand::WriteMemoryByAddress => {}
            UDSCommand::TesterPresent => {}
            UDSCommand::RequestFileTransfer => {}
            UDSCommand::ControlDTCSetting => {}
            UDSCommand::LinkControl => {}
        }
        "--TODO--".into()
    }
}

impl ToString for UDSCommand {
    fn to_string(&self) -> String {
        self.get_name()
    }
}

impl Into<u8> for UDSCommand {
    fn into(self) -> u8 {
        match &self {
            UDSCommand::DiagnosticSessionControl => 0x10,
            UDSCommand::ECUReset => 0x11,
            UDSCommand::ClearDTCInformation => 0x14,
            UDSCommand::ReadDTCInformation => 0x19,
            UDSCommand::ReadDataByID => 0x22,
            UDSCommand::ReadMemoryByAddress => 0x23,
            UDSCommand::ReadScalingDataById => 0x24,
            UDSCommand::SecurityAccess => 0x27,
            UDSCommand::CommunicationControl => 0x28,
            UDSCommand::Authentication => 0x29,
            UDSCommand::ReadDataByPeriodicID => 0x2A,
            UDSCommand::DynamicDefineDataId => 0x2C,
            UDSCommand::WriteDataByID => 0x2E,
            UDSCommand::IOCTLById => 0x2F,
            UDSCommand::RoutineControl => 0x31,
            UDSCommand::RequestDownload => 0x34,
            UDSCommand::RequestUpload => 0x35,
            UDSCommand::TransferData => 0x36,
            UDSCommand::TransferExit => 0x37,
            UDSCommand::WriteMemoryByAddress => 0x3D,
            UDSCommand::TesterPresent => 0x3E,
            UDSCommand::RequestFileTransfer => 0x3F,
            UDSCommand::ControlDTCSetting => 0x85,
            UDSCommand::LinkControl => 0x87,
        }
    }
}

impl ECUCommand for UDSCommand {
    fn get_caution_level(&self) -> CautionLevel {
        match &self {
            UDSCommand::DiagnosticSessionControl => CautionLevel::Warn,
            UDSCommand::ECUReset => CautionLevel::Alert,
            UDSCommand::ClearDTCInformation => CautionLevel::None,
            UDSCommand::ReadDTCInformation => CautionLevel::None,
            UDSCommand::ReadDataByID => CautionLevel::None,
            UDSCommand::ReadMemoryByAddress => CautionLevel::Warn,
            UDSCommand::ReadScalingDataById => CautionLevel::Warn,
            UDSCommand::SecurityAccess => CautionLevel::Alert,
            UDSCommand::CommunicationControl => CautionLevel::Warn,
            UDSCommand::Authentication => CautionLevel::Alert,
            UDSCommand::ReadDataByPeriodicID => CautionLevel::Warn,
            UDSCommand::DynamicDefineDataId => CautionLevel::Alert,
            UDSCommand::WriteDataByID => CautionLevel::Alert,
            UDSCommand::IOCTLById => CautionLevel::Alert,
            UDSCommand::RoutineControl => CautionLevel::Warn,
            UDSCommand::RequestDownload => CautionLevel::Alert,
            UDSCommand::RequestUpload => CautionLevel::Alert,
            UDSCommand::TransferData => CautionLevel::Alert,
            UDSCommand::TransferExit => CautionLevel::Alert,
            UDSCommand::WriteMemoryByAddress => CautionLevel::Alert,
            UDSCommand::TesterPresent => CautionLevel::None,
            UDSCommand::RequestFileTransfer => CautionLevel::Alert,
            UDSCommand::ControlDTCSetting => CautionLevel::Warn,
            UDSCommand::LinkControl => CautionLevel::Warn
        }
    }

    fn get_cmd_list() -> Vec<Self> {
        vec![
            //Self::DiagnosticSessionControl,
            Self::ECUReset,
            Self::ClearDTCInformation,
            Self::ReadDTCInformation,
            Self::ReadDataByID,
            Self::ReadMemoryByAddress,
            Self::ReadScalingDataById,
            Self::SecurityAccess,
            Self::CommunicationControl,
            Self::Authentication,
            Self::ReadDataByPeriodicID,
            Self::DynamicDefineDataId,
            Self::WriteDataByID,
            Self::IOCTLById,
            Self::RoutineControl,
            Self::RequestDownload,
            Self::RequestUpload,
            Self::TransferData,
            Self::TransferExit,
            Self::WriteMemoryByAddress,
            //Self::TesterPresent,
            Self::RequestFileTransfer,
            Self::ControlDTCSetting,
            Self::LinkControl,
        ]
    }
}

#[derive(Copy, Debug, Clone, Eq, PartialOrd, PartialEq)]
/// All possible UDS Negative responses an ECU can return
/// when trying to run a command
pub enum UDSNegativeCode {
    GeneralReject,
    ServiceNotSupported,
    SubFunctionNotSupported,
    IncorrectMessageLength,
    ResponseTooLong,
    BusyRepeatRequest,
    ConditionsNotCorrect,
    RequestSequenceError,
    RequestOutOfRange,
    SecurityAccessDenied,
    InvalidKey,
    ExceedNumberOfAttempts,
    RequiredTimeDelayNotExpired,
    UploadDownloadNotAccepted,
    TransferDataSuspended,
    GeneralProgrammingFailure,
    WrongBlockSequenceCounter,
    ResponsePending,
    SubFunctionNotSupportedActiveSession,
    ServiceNotSupportedActiveSession,
    RpmTooHigh,
    RpmTooLow,
    EngineIsRunning,
    EngineIsNotRunning,
    EngineRunTimeTooLow,
    TempTooHigh,
    TempTooLow,
    SpeedTooHigh,
    SpeedTooLow,
    ThrottleTooHigh,
    ThrottleTooLow,
    TransmissionNotInNeutral,
    TransmissionNotInGear,
    BrakeNotApplied,
    ShifterNotInPark,
    TorqueConverterClutchLocked,
    VoltageTooHigh,
    VoltageTooLow,
    ReservedSpecificConditionsIncorrect,
    NoResponseSubnetComponent,
    FailurePreventsExecutionOfRequestedAction,
    Reserved(u8)
}

impl CommandError for UDSNegativeCode {
    fn get_text(&self) -> String {
        match &self {
            UDSNegativeCode::GeneralReject => "General reject",
            UDSNegativeCode::ServiceNotSupported => "Service is not supported",
            UDSNegativeCode::SubFunctionNotSupported => "Sub function is not supported",
            UDSNegativeCode::IncorrectMessageLength => "Incorrect message length",
            UDSNegativeCode::ResponseTooLong => "Response is too long for transport protocol",
            UDSNegativeCode::BusyRepeatRequest => "ECU is busy",
            UDSNegativeCode::ConditionsNotCorrect => "Conditions are not correct",
            UDSNegativeCode::RequestSequenceError => "Message sequence is not correct",
            UDSNegativeCode::RequestOutOfRange => "Requested data is out of range",
            UDSNegativeCode::SecurityAccessDenied => "Security access is denied",
            UDSNegativeCode::InvalidKey => "Invalid key",
            UDSNegativeCode::ExceedNumberOfAttempts => "Exceeded number of access attempts",
            UDSNegativeCode::RequiredTimeDelayNotExpired => "Security timeout has not expired",
            UDSNegativeCode::UploadDownloadNotAccepted => "Upload/Download is not accepted",
            UDSNegativeCode::TransferDataSuspended => "Transfer operation halted",
            UDSNegativeCode::GeneralProgrammingFailure => "Programming error",
            UDSNegativeCode::WrongBlockSequenceCounter => "Error in block sequence",
            UDSNegativeCode::ResponsePending => "ECU is responding. Wait",
            UDSNegativeCode::SubFunctionNotSupportedActiveSession => "Function is not supported in this diagnostic session",
            UDSNegativeCode::ServiceNotSupportedActiveSession => "Service is not supported in this diagnostic session",
            UDSNegativeCode::RpmTooHigh => "Engine RPM is too high",
            UDSNegativeCode::RpmTooLow => "Engine RPM is too low",
            UDSNegativeCode::EngineIsRunning => "Engine is running",
            UDSNegativeCode::EngineIsNotRunning => "Engine is not running",
            UDSNegativeCode::EngineRunTimeTooLow => "Engine has not been on for long enough",
            UDSNegativeCode::TempTooHigh => "Engine temperature is too high",
            UDSNegativeCode::TempTooLow => "Engine temperature is too low",
            UDSNegativeCode::SpeedTooHigh => "Vehicle speed is too high",
            UDSNegativeCode::SpeedTooLow => "Vehicle speed is too low",
            UDSNegativeCode::ThrottleTooHigh => "Throttle is too high",
            UDSNegativeCode::ThrottleTooLow => "Throttle is too low",
            UDSNegativeCode::TransmissionNotInNeutral => "Transmission is not in neutral",
            UDSNegativeCode::TransmissionNotInGear => "Transmission is not in gear",
            UDSNegativeCode::BrakeNotApplied => "Brake is not applied",
            UDSNegativeCode::ShifterNotInPark => "Transmission is not in park",
            UDSNegativeCode::TorqueConverterClutchLocked => "Torque converter clutch is locked",
            UDSNegativeCode::VoltageTooHigh => "Voltage is too high",
            UDSNegativeCode::VoltageTooLow => "Voltage is too low",
            UDSNegativeCode::ReservedSpecificConditionsIncorrect => "Failure condition for test met",
            UDSNegativeCode::NoResponseSubnetComponent => "Subnet component did not respond",
            UDSNegativeCode::FailurePreventsExecutionOfRequestedAction => "",
            UDSNegativeCode::Reserved(b) => return format!("Reserved error 0x{:02X}", b)
        }.into()
    }

    fn get_help(&self) -> Option<String> {
        match &self {
            UDSNegativeCode::GeneralReject => {}
            UDSNegativeCode::ServiceNotSupported => {}
            UDSNegativeCode::SubFunctionNotSupported => {}
            UDSNegativeCode::IncorrectMessageLength => {}
            UDSNegativeCode::ResponseTooLong => {}
            UDSNegativeCode::BusyRepeatRequest => {}
            UDSNegativeCode::ConditionsNotCorrect => {}
            UDSNegativeCode::RequestSequenceError => {}
            UDSNegativeCode::RequestOutOfRange => {}
            UDSNegativeCode::SecurityAccessDenied => {}
            UDSNegativeCode::InvalidKey => {}
            UDSNegativeCode::ExceedNumberOfAttempts => {}
            UDSNegativeCode::RequiredTimeDelayNotExpired => {}
            UDSNegativeCode::UploadDownloadNotAccepted => {}
            UDSNegativeCode::TransferDataSuspended => {}
            UDSNegativeCode::GeneralProgrammingFailure => {}
            UDSNegativeCode::WrongBlockSequenceCounter => {}
            UDSNegativeCode::ResponsePending => {}
            UDSNegativeCode::SubFunctionNotSupportedActiveSession => {}
            UDSNegativeCode::ServiceNotSupportedActiveSession => {}
            UDSNegativeCode::RpmTooHigh => {}
            UDSNegativeCode::RpmTooLow => {}
            UDSNegativeCode::EngineIsRunning => {}
            UDSNegativeCode::EngineIsNotRunning => {}
            UDSNegativeCode::EngineRunTimeTooLow => {}
            UDSNegativeCode::TempTooHigh => {}
            UDSNegativeCode::TempTooLow => {}
            UDSNegativeCode::SpeedTooHigh => {}
            UDSNegativeCode::SpeedTooLow => {}
            UDSNegativeCode::ThrottleTooHigh => {}
            UDSNegativeCode::ThrottleTooLow => {}
            UDSNegativeCode::TransmissionNotInNeutral => {}
            UDSNegativeCode::TransmissionNotInGear => {}
            UDSNegativeCode::BrakeNotApplied => {}
            UDSNegativeCode::ShifterNotInPark => {}
            UDSNegativeCode::TorqueConverterClutchLocked => {}
            UDSNegativeCode::VoltageTooHigh => {}
            UDSNegativeCode::VoltageTooLow => {}
            UDSNegativeCode::ReservedSpecificConditionsIncorrect => {}
            UDSNegativeCode::NoResponseSubnetComponent => {}
            UDSNegativeCode::FailurePreventsExecutionOfRequestedAction => {}
            UDSNegativeCode::Reserved(_) => {}
        }
        None
    }

    fn from_byte<'a>(b: u8) -> Self where Self: Sized {
        match b {
            // Reserved
            0x10 => Self::GeneralReject,
            0x11 => Self::ServiceNotSupported,
            0x12 => Self::SubFunctionNotSupported,
            0x13 => Self::IncorrectMessageLength,
            0x14 => Self::ResponseTooLong,
            // Reserved
            0x21 => Self::BusyRepeatRequest,
            0x22 => Self::ConditionsNotCorrect,
            // Reserved
            0x24 => Self::RequestSequenceError,
            0x25 => Self::NoResponseSubnetComponent,
            0x26 => Self::FailurePreventsExecutionOfRequestedAction,
            // Reserved
            0x31 => Self::RequestOutOfRange,
            //Reserved
            0x33 => Self::SecurityAccessDenied,
            // Reserved
            0x35 => Self::InvalidKey,
            0x36 => Self::ExceedNumberOfAttempts,
            0x37 => Self::RequiredTimeDelayNotExpired,
            // Reserved data link security (38-4F)
            // Reserved (50-6F)
            0x70 => Self::UploadDownloadNotAccepted,
            0x71 => Self::TransferDataSuspended,
            0x72 => Self::GeneralProgrammingFailure,
            0x73 => Self::WrongBlockSequenceCounter,
            // Reserved
            0x78 => Self::ResponsePending,
            // Reserved
            0x7E => Self::SubFunctionNotSupportedActiveSession,
            0x7F => Self::ServiceNotSupportedActiveSession,
            // Reserved
            0x81 => Self::RpmTooHigh,
            0x82 => Self::RpmTooLow,
            0x83 => Self::EngineIsRunning,
            0x84 => Self::EngineIsNotRunning,
            0x85 => Self::EngineRunTimeTooLow,
            0x86 => Self::TempTooHigh,
            0x87 => Self::TempTooLow,
            0x88 => Self::SpeedTooHigh,
            0x89 => Self::SpeedTooLow,
            0x8A => Self::ThrottleTooHigh,
            0x8B => Self::ThrottleTooLow,
            0x8C => Self::TransmissionNotInNeutral,
            0x8D => Self::TransmissionNotInGear,
            // Reserved
            0x8F => Self::BrakeNotApplied,
            0x90 => Self::ShifterNotInPark,
            0x91 => Self::TorqueConverterClutchLocked,
            0x92 => Self::VoltageTooHigh,
            0x93 => Self::VoltageTooLow,
            // Reserved
            _ => Self::Reserved(b)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UDSECU {
    iso_tp_settings: ISO15765Config,
    should_run: Arc<AtomicBool>,
    last_error: Arc<RwLock<Option<ProtocolError>>>,
    cmd_tx: Sender<(u8, Vec<u8>, bool)>,
    cmd_rx: Arc<Receiver<ProtocolResult<Vec<u8>>>>,
    curr_session_type: Arc<RwLock<DiagSession>>,
    send_id: u32,
    cmd_mutex: Arc<Mutex<()>>
}

impl UDSECU {

    pub fn clear_errors(&self) -> std::result::Result<(), ProtocolError> {
        self.run_command(UDSCommand::ClearDTCInformation.into(), &[0xFF, 0x00])?;
        Ok(())
    }

    fn set_diag_session_mode(&mut self, mode: DiagSession) -> std::result::Result<(), ProtocolError> {
        match diag_session_control::set_diag_session(&self, mode) {
            Ok(_) => {
                *self.curr_session_type.write().unwrap() = mode; // Switch diagnostic modes!
                Ok(())
            },
            Err(e) => {
                *self.curr_session_type.write().unwrap() = DiagSession::Default; // Assume normal if something happens
                Err(e)
            }
        }
    }

    pub fn get_session_type(&self) -> DiagSession {
        *self.curr_session_type.read().unwrap()
    }
}



impl ProtocolServer for UDSECU {
    type Command = UDSCommand;
    type Error = UDSNegativeCode;
    fn start_diag_session(mut comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> ProtocolResult<Self> {
        comm_server.open_iso15765_interface(500_000, false).map_err(ProtocolError::CommError)?;
        comm_server.configure_iso15765(cfg).map_err(ProtocolError::CommError)?;

        let should_run = Arc::new(AtomicBool::new(true));
        let should_run_t = should_run.clone();

        let last_error = Arc::new(RwLock::new(None));
        let last_error_t = last_error.clone();

        let (channel_tx_sender, channel_tx_receiver): (Sender<(u8, Vec<u8>, bool)>, Receiver<(u8, Vec<u8>, bool)>) = mpsc::channel();
        let (channel_rx_sender, channel_rx_receiver): (Sender<ProtocolResult<Vec<u8>>>, Receiver<ProtocolResult<Vec<u8>>>) = mpsc::channel();

        let session_type = Arc::new(RwLock::new(DiagSession::Default));
        let session_type_t = session_type.clone();

        // Enter extended diagnostic session (Full features)
        let s_id = cfg.send_id;
        std::thread::spawn(move || {
            println!("Diag server start!");
            let mut timer = Instant::now();
            while should_run_t.load(Relaxed) {
                if let Ok(data) = channel_tx_receiver.try_recv() {
                    let res = Self::run_command_iso_tp(comm_server.as_ref(), s_id, data.0, &data.1, data.2);
                    if channel_rx_sender.send(res).is_err() {
                        *last_error_t.write().unwrap() = Some(ProtocolError::CustomError("Sender channel died".into()));
                        break
                    }
                }
                if timer.elapsed().as_millis() >= 2000 && *session_type_t.read().unwrap() != DiagSession::Default {
                    if Self::run_command_iso_tp(comm_server.as_ref(), s_id, UDSCommand::TesterPresent.into(), &[0x01], true).is_err() {
                        println!("Lost connection with ECU!");
                    }
                    timer = Instant::now();
                }
                std::thread::sleep(std::time::Duration::from_micros(100))
            }
            println!("Diag server stop!");
            comm_server.close_iso15765_interface();
        });

        let mut ecu = UDSECU {
            iso_tp_settings: *cfg,
            should_run,
            last_error,
            cmd_tx: channel_tx_sender,
            cmd_rx: Arc::new(channel_rx_receiver),
            send_id: cfg.send_id,
            curr_session_type: session_type, // Assumed,
            cmd_mutex: Arc::new(Mutex::new(()))
        };

        if let Err(e) = ecu.set_diag_session_mode(DiagSession::Extended) {
            ecu.should_run.store(false, Relaxed);
            return Err(e)
        }
        Ok(ecu)
    }

    fn exit_diag_session(&mut self) {
        self.should_run.store(false, Relaxed);
    }

    fn run_command(&self, cmd: u8, args: &[u8]) -> ProtocolResult<Vec<u8>> {
        let _guard = self.cmd_mutex.lock().unwrap(); // We are allowed to send / receive!
        if self.cmd_tx.send((cmd, Vec::from(args), true)).is_err() {
            return Err(ProtocolError::CustomError("Channel Tx failed".into()))
        }
        let resp = self.cmd_rx.recv().unwrap()?;
        if resp[0] == 0x7F {
            let neg_code = UDSNegativeCode::from_byte(resp[2]);
            Err(ProtocolError::ProtocolError(Box::new(neg_code)))
        } else {
            Ok(resp)
        }
    }

    fn read_errors(&self) -> ProtocolResult<Vec<DTC>> {
        todo!()
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

    fn run_command_iso_tp(server: &dyn ComServer, send_id: u32, cmd: u8, args: &[u8], receive_require: bool) -> Result<Vec<u8>, ProtocolError> {
        let mut data = crate::commapi::comm_api::ISO15765Data {
            id: send_id,
            data: vec![cmd],
            pad_frame: false,
        };
        data.data.extend_from_slice(args);
        if !receive_require {
            server.send_iso15765_data(&[data], 0).map(|_| vec![]).map_err(ProtocolError::CommError)
        } else {
            // Await max 1 second for response
            let res = server.send_receive_iso15765(data, 1000, 1)?;
            if res.is_empty() {
                return Err(ProtocolError::Timeout)
            }
            let mut tmp_res = res[0].data.clone();
            if tmp_res[0] == 0x7F && tmp_res[2] == 0x78 { // ResponsePending
                println!("UDS - ECU is processing request - Waiting!");
                let start = Instant::now();
                while start.elapsed().as_millis() < 1000 {
                    // ECU is sending a response, but its busy right now. just gotta wait for the ECU to give us its response!
                    if let Some(msg) = server.read_iso15765_packets(0, 1)?.get(0) {
                        tmp_res = msg.data.clone();
                    }
                }
            }
            if tmp_res[0] == 0x7F {
                // Still error :(
                Err(ProtocolError::ProtocolError(Box::new(Self::Error::from_byte(tmp_res[2]))))
            } else if tmp_res[0] == (cmd + 0x40) {
                Ok(tmp_res)
            } else {
                eprintln!("UDS - Command response did not match request? Send: {:02X} - Recv: {:02X}", cmd, tmp_res[0]);
                Err(ProtocolError::Timeout)
            }
        }
    }
}