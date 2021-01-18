use std::fmt::Display;

use comm_api::ISO15765Config;

use super::comm_api::{self, ComServer};

pub mod uds;
pub mod obd2;
pub mod vin;
pub mod kwp2000;

#[derive(Debug, Clone)]
pub enum ProtocolError {
    CommError(comm_api::ComServerError),
    ProtocolError(String),
    Timeout,
}

type ProtocolResult<T> = std::result::Result<T, ProtocolError>;

pub trait Selectable {
    fn get_byte(&self) -> u8;
    fn get_desc(&self) -> String;
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

pub trait CommandLevel {
    fn get_caution_level(&self) -> CautionLevel;
}

pub trait CommandError {
    fn get_desc(&self) -> String;
    fn get_name(&self) -> String;
    fn get_byte(&self) -> u8;
    fn from_byte(b: u8) -> Self;
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

pub trait ProtocolServer : Clone {
    type Command: Selectable + CommandLevel;
    
    fn start_diag_session(comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> std::result::Result<Self, ProtocolError>;
    fn exit_diag_session(&mut self);
    fn run_command(&self, cmd: Self::Command, args: &[u8], max_timeout_ms: u128) -> ProtocolResult<Vec<u8>>;

    fn read_errors(&self) -> std::result::Result<Vec<DTC>, ProtocolError>;
}