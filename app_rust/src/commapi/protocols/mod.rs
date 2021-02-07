use std::fmt::Display;

use comm_api::ISO15765Config;
use kwp2000::KWP2000ECU;

use super::comm_api::{self, ComServer};

pub mod uds;
pub mod obd2;
pub mod vin;
pub mod kwp2000;

#[derive(Debug)]
pub enum ProtocolError {
    CommError(comm_api::ComServerError),
    ProtocolError(Box<dyn CommandError>),
    CustomError(String),
    Timeout,
}

unsafe impl Send for ProtocolError{}
unsafe impl Sync for ProtocolError{}

impl ProtocolError {
    pub fn get_text(&self) -> String {
        match self {
            ProtocolError::CommError(e) => e.to_string(),
            ProtocolError::ProtocolError(e) => e.get_text(),
            ProtocolError::Timeout => "Communication timeout".into(),
            ProtocolError::CustomError(s) => s.clone()
        }
    }
}

pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

pub trait Selectable {
    fn get_byte(&self) -> u8;
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
    Alert = 2
}

pub trait ECUCommand : Selectable {
    fn get_caution_level(&self) -> CautionLevel;
}

pub trait CommandError {
    fn get_text(&self) -> String;
    fn get_help(&self) -> Option<String>;
    fn from_byte(b: u8) -> Self where Self: Sized;
}

impl std::fmt::Debug for Box<dyn CommandError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CMDError {}", self.get_text())
    }
}

pub struct DTC {
    pub (crate) error: String,
    pub (crate) present: bool,
    pub (crate) stored: bool,
    pub (crate) check_engine_on: bool,
}

impl Display for DTC {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - Present?: {}, In memory?: {}, Check engine light on?: {}", self.error, self.present, self.stored, self.check_engine_on)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DiagProtocol {
    KWP2000,
    UDS
}

#[derive(Debug, Clone)]
pub enum DiagServer {
    KWP2000(KWP2000ECU),
    UDS
}

impl DiagServer {
    pub fn new(comm_server: Box<dyn ComServer>, cfg: &ISO15765Config, protocol: DiagProtocol) -> ProtocolResult<Self> {
        Ok(match protocol {
            KWP2000 => Self::KWP2000(KWP2000ECU::start_diag_session(comm_server, cfg)?),
            UDS => todo!("Diag protocol UDS not implemented")
        })
    }

    pub fn get_name<'a>(&self) -> &'a str {
        match self {
            Self::KWP2000(_) => "KWP2000",
            Self::UDS => "UDS"
        }
    }

    pub fn kill_diag_server(&mut self) {
        match self {
            Self::KWP2000(s) => s.exit_diag_session(),
            Self::UDS => todo!()
        }
    }

    pub fn run_cmd(&mut self, cmd: u8, args: &[u8], max_timeout_ms: u128) -> ProtocolResult<Vec<u8>> {
        match self {
            Self::KWP2000(s) => s.run_command(cmd, args, max_timeout_ms),
            Self::UDS => todo!()
        }
    }

    pub fn read_errors(&self) -> ProtocolResult<Vec<DTC>> {
        match self {
            Self::KWP2000(s) => s.read_errors(),
            Self::UDS => todo!()
        }
    }

    pub fn clear_errors(&self) -> ProtocolResult<()> {
        match self {
            Self::KWP2000(s) => s.clear_errors(),
            Self::UDS => todo!()
        }
    }
}

impl Drop for DiagServer {
    fn drop(&mut self) {
        println!("Drop for diagserver called!");
        self.kill_diag_server()
    }
}

pub trait ProtocolServer : Clone {
    type Command: Selectable + ECUCommand;
    fn start_diag_session(comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> ProtocolResult<Self>;
    fn exit_diag_session(&mut self);
    fn run_command(&self, cmd: u8, args: &[u8], max_timeout_ms: u128) -> ProtocolResult<Vec<u8>>;
    fn read_errors(&self) -> ProtocolResult<Vec<DTC>>;
    fn is_in_diag_session(&self) -> bool;
    fn get_last_error(&self) -> Option<String>;
}