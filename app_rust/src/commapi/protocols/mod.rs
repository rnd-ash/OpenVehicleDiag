use std::{fmt::Display, time::Instant};

use comm_api::{ComServerError, ISO15765Config};
use kwp2000::KWP2000ECU;
use uds::UDSECU;

use self::kwp2000::read_ecu_identification;

use super::comm_api::{self, ComServer, ISO15765Data};

pub mod kwp2000;
pub mod obd2;
pub mod uds;
pub mod vin;

#[derive(Debug)]
pub enum ProtocolError {
    CommError(comm_api::ComServerError),
    ProtocolError(Box<dyn CommandError>),
    CustomError(String),
    InvalidResponseSize { expect: usize, actual: usize },
    Timeout,
}

impl ProtocolError {
    pub fn is_timeout(&self) -> bool {
        match &self {
            ProtocolError::CommError(_) => false,
            ProtocolError::ProtocolError(_) => false,
            ProtocolError::CustomError(_) => false,
            ProtocolError::InvalidResponseSize { expect, actual } => false,
            ProtocolError::Timeout => true,
        }
    }
}

impl From<ComServerError> for ProtocolError {
    fn from(x: ComServerError) -> Self {
        ProtocolError::CommError(x)
    }
}

unsafe impl Send for ProtocolError {}
unsafe impl Sync for ProtocolError {}

impl ProtocolError {
    pub fn get_text(&self) -> String {
        match self {
            ProtocolError::CommError(e) => e.to_string(),
            ProtocolError::ProtocolError(e) => e.get_desc(),
            ProtocolError::Timeout => "Communication timeout".into(),
            ProtocolError::CustomError(s) => s.clone(),
            ProtocolError::InvalidResponseSize { expect, actual } => {
                format!("Expected {} bytes, got {} bytes", expect, actual)
            }
        }
    }
}

pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

pub trait Selectable: Into<u8> {
    fn get_desc(&self) -> String;
    fn get_name(&self) -> String;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CautionLevel {
    // This has no adverse effects on the ECU
    None = 0,
    // This might cause unpredictable behavior
    Warn = 1,
    // Danger Zone - Do not run this unless you know what you are doing!
    Alert = 2,
}

pub trait ECUCommand: Selectable {
    fn get_caution_level(&self) -> CautionLevel;
    fn get_cmd_list() -> Vec<Self>;
}

pub trait CommandError {
    fn get_desc(&self) -> String;
    fn get_help(&self) -> Option<String>;
    fn from_byte(b: u8) -> Self
    where
        Self: Sized;
}

impl std::fmt::Debug for Box<dyn CommandError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CMDError {}", self.get_desc())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DTCState {
    None,
    Stored,
    Pending,
    Permanent
}

pub struct DTC {
    pub(crate) error: String,
    pub(crate) state: DTCState,
    pub(crate) check_engine_on: bool,
}

impl Display for DTC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - State: {:?}, Check engine light on?: {}",
            self.error, self.state, self.check_engine_on
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DiagProtocol {
    KWP2000,
    UDS,
}

#[derive(Debug, Clone)]
pub enum DiagServer {
    KWP2000(KWP2000ECU),
    UDS(UDSECU),
}

impl DiagServer {
    pub fn new(
        comm_server: Box<dyn ComServer>,
        cfg: &ISO15765Config,
        global_test_present_addr: Option<u32>,
        protocol: DiagProtocol,
    ) -> ProtocolResult<Self> {
        Ok(match protocol {
            KWP2000 => Self::KWP2000(KWP2000ECU::start_diag_session(comm_server, cfg, global_test_present_addr)?),
            UDS => Self::UDS(UDSECU::start_diag_session(comm_server, cfg, global_test_present_addr)?),
        })
    }

    pub fn get_name<'a>(&self) -> &'a str {
        match self {
            Self::KWP2000(_) => "KWP2000",
            Self::UDS(_) => "UDS",
        }
    }

    pub fn kill_diag_server(&mut self) {
        match self {
            Self::KWP2000(s) => s.exit_diag_session(),
            Self::UDS(s) => s.exit_diag_session(),
        }
    }

    pub fn run_cmd(&mut self, cmd: u8, args: &[u8]) -> ProtocolResult<Vec<u8>> {
        match self {
            Self::KWP2000(s) => s.run_command(cmd, args),
            Self::UDS(s) => s.run_command(cmd, args),
        }
    }

    pub fn read_errors(&self) -> ProtocolResult<Vec<DTC>> {
        match self {
            Self::KWP2000(s) => s.read_errors(),
            Self::UDS(s) => s.read_errors(),
        }
    }

    pub fn clear_errors(&self) -> ProtocolResult<()> {
        match self {
            Self::KWP2000(s) => s.clear_errors(),
            Self::UDS(s) => s.clear_errors(),
        }
    }

    pub fn get_variant_id(&self) -> ProtocolResult<u16> {
        match self {
            Self::KWP2000(s) => read_ecu_identification::read_dcx_mmc_id(&s).map(|x| x.diag_information),
            Self::UDS(s) => Err(ProtocolError::CustomError("Not implemented".into())), // TODO
        }
    }
}

impl Drop for DiagServer {
    fn drop(&mut self) {
        println!("Drop for Diag Server called!");
        self.kill_diag_server()
    }
}

pub trait ProtocolServer: Sized {
    type Command: Selectable + ECUCommand;
    type Error: CommandError + 'static;
    fn start_diag_session(
        comm_server: Box<dyn ComServer>,
        cfg: &ISO15765Config,
        global_tester_present_addr: Option<u32>,
    ) -> ProtocolResult<Self>;
    fn exit_diag_session(&mut self);
    fn run_command(&self, cmd: u8, args: &[u8]) -> ProtocolResult<Vec<u8>>;
    fn read_errors(&self) -> ProtocolResult<Vec<DTC>>;
    fn is_in_diag_session(&self) -> bool;
    fn get_last_error(&self) -> Option<String>;

    fn run_command_iso_tp(
        server: &dyn ComServer,
        send_id: u32,
        cmd: u8,
        args: &[u8],
        receive_require: bool,
    ) -> std::result::Result<Vec<u8>, ProtocolError> {
        let mut data = ISO15765Data {
            id: send_id,
            data: vec![cmd],
            pad_frame: false,
            ext_addressing: true,
        };
        data.data.extend_from_slice(args);
        if !receive_require {
            server
                .send_iso15765_data(&[data], 0)
                .map(|_| vec![])
                .map_err(ProtocolError::CommError)
        } else {
            // Await max 1 second for response
            let res = server.send_receive_iso15765(data, 1000, 1)?;
            if res.is_empty() {
                return Err(ProtocolError::Timeout);
            }
            let mut tmp_res = res[0].data.clone();
            if tmp_res[0] == 0x7F && tmp_res[2] == 0x78 {
                // ResponsePending
                println!("KWP2000 - ECU is processing request - Waiting!");
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
                Err(ProtocolError::ProtocolError(Box::new(
                    Self::Error::from_byte(tmp_res[2]),
                )))
            } else if tmp_res[0] == (cmd + 0x40) {
                Ok(tmp_res)
            } else {
                eprintln!(
                    "KWP2000 - Command response did not match request? Send: {:02X} - Recv: {:02X}",
                    cmd, tmp_res[0]
                );
                Err(ProtocolError::Timeout)
            }
        }
    }
}
