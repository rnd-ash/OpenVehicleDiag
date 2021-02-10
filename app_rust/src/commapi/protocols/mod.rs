use std::{fmt::Display, time::Instant};

use comm_api::{ComServerError, ISO15765Config};
use kwp2000::KWP2000ECU;

use super::comm_api::{self, ComServer, ISO15765Data};

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

impl From<ComServerError> for ProtocolError {
    fn from(x: ComServerError) -> Self {
        ProtocolError::CommError(x)
    }
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
    Alert = 2
}

pub trait ECUCommand : Selectable {
    fn get_caution_level(&self) -> CautionLevel;
}

pub trait CommandError {
    fn get_text(&self) -> String;
    fn get_help(&self) -> Option<String>;
    fn from_byte<'a>(b: u8) -> Self where Self: Sized;
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

pub trait ProtocolServer: Sized {
    type Command: Selectable + ECUCommand;
    type Error: CommandError + 'static;
    fn start_diag_session(comm_server: Box<dyn ComServer>, cfg: &ISO15765Config) -> ProtocolResult<Self>;
    fn exit_diag_session(&mut self);
    fn run_command(&self, cmd: u8, args: &[u8], max_timeout_ms: u128) -> ProtocolResult<Vec<u8>>;
    fn read_errors(&self) -> ProtocolResult<Vec<DTC>>;
    fn is_in_diag_session(&self) -> bool;
    fn get_last_error(&self) -> Option<String>;

    fn run_command_isotp(server: &dyn ComServer, send_id: u32, cmd: u8, args: &[u8], receive_require: bool) -> std::result::Result<Vec<u8>, ProtocolError> {
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
                Err(ProtocolError::ProtocolError(Box::new(Self::Error::from_byte(tmp_res[2]))))
            } else if tmp_res[0] == (cmd + 0x40) {
                Ok(tmp_res)
            } else {
                eprintln!("KWP2000 - Command response did not match request? Send: {:02X} - Recv: {:02X}", cmd, tmp_res[0]);
                Err(ProtocolError::Timeout)
            }
        }
    }
}