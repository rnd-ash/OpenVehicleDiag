use std::{sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}}, vec};

use commapi::protocols::ProtocolError;

use crate::commapi;

use self::{service01::Service01, service03::Service03, service09::Service09};

use super::{CommandError, ECUCommand, ProtocolResult, ProtocolServer, Selectable};

pub mod service01;
pub mod service02;
pub mod service03;
pub mod service04;
pub mod service09;

pub type OBDError<T> = ProtocolResult<T>;

// Helper function to get bits from byte array, in order MSB to LSB
pub (crate) fn get_obd_bits(src: &[u8]) -> Vec<bool> {
    let mut res = Vec::new();
    for byte in src {
        let mut mask: u8 = 0b10000000;
        for _ in 0..8 {
            res.push(byte & mask != 0x00);
            mask = mask >> 1;
        }
    }
    res
}

trait ObdService where Self: Sized {
    fn init(s: &ObdServer) -> Option<Self>;
}



#[derive(Debug, Clone, Copy)]
pub enum ObdError {
    CmdNotSupported
}

impl CommandError for ObdError {
    fn get_desc(&self) -> String {
        "OBD Command not supported by ECU".into()
    }

    fn get_help(&self) -> Option<String> {
        Some("OBD Command not supported".into())
    }

    fn from_byte(b: u8) -> Self
    where
        Self: Sized {
        Self::CmdNotSupported
    }
}

#[derive(Debug, Copy, Clone)]
pub enum OBDCmd{
    Service01,
    Service02,
    Service03,
    Service04,
    Service05,
    Service06,
    Service07,
    Service08,
    Service09,
    Service0A,
}

impl Selectable for OBDCmd {
    fn get_desc(&self) -> String {
        match &self {
            OBDCmd::Service01 => "Show current data",
            OBDCmd::Service02 => "Show freeze frame data",
            OBDCmd::Service03 => "Show DTCs",
            OBDCmd::Service04 => "Clear DTCs",
            OBDCmd::Service05 => "Test results, O2 sensor monitoring (Non CAN)",
            OBDCmd::Service06 => "Test results, 02 sensor monitoring (CAN only)",
            OBDCmd::Service07 => "Show pending DTCs",
            OBDCmd::Service08 => "Control operation of on-board systems",
            OBDCmd::Service09 => "Request vehicle information",
            OBDCmd::Service0A => "Permanent DTCs",
        }.into()
    }

    fn get_name(&self) -> String {
        format!("{:?}", &self)
    }
}

impl Into<u8> for OBDCmd {
    fn into(self) -> u8 {
        match &self {
            OBDCmd::Service01 => 0x01,
            OBDCmd::Service02 => 0x02,
            OBDCmd::Service03 => 0x03,
            OBDCmd::Service04 => 0x04,
            OBDCmd::Service05 => 0x05,
            OBDCmd::Service06 => 0x06,
            OBDCmd::Service07 => 0x07,
            OBDCmd::Service08 => 0x08,
            OBDCmd::Service09 => 0x09,
            OBDCmd::Service0A => 0x0A,
        }
    }
}

impl ECUCommand for OBDCmd {
    fn get_caution_level(&self) -> super::CautionLevel {
        super::CautionLevel::None // Always for OBD - Read only protocol so nothing bad can happen :)
    }

    fn get_cmd_list() -> Vec<Self> {
        vec![
            OBDCmd::Service01,
            OBDCmd::Service02,
            OBDCmd::Service03,
            OBDCmd::Service04,
            OBDCmd::Service05,
            OBDCmd::Service06,
            OBDCmd::Service07,
            OBDCmd::Service08,
            OBDCmd::Service09,
            OBDCmd::Service0A,
        ]
    }
}

#[deny(Debug, Clone)]
pub struct ObdServer {
    should_run: Arc<AtomicBool>,
    cmd_tx: Sender<(u8, Vec<u8>, bool)>,
    cmd_rx: Arc<Receiver<ProtocolResult<Vec<u8>>>>,
    cmd_mutex: Arc<Mutex<()>>,
}

impl ProtocolServer for ObdServer {
    type Command = OBDCmd;

    type Error = ObdError;

    fn start_diag_session(
        mut comm_server: Box<dyn crate::commapi::comm_api::ComServer>,
        cfg: &crate::commapi::comm_api::ISO15765Config,
        _global_tester_present_addr: Option<u32>, // Should always be none for OBD
    ) -> super::ProtocolResult<Self> {
        comm_server.open_iso15765_interface(cfg.baud, false, false)?; // For OBD this should work
        comm_server.add_iso15765_filter(cfg.recv_id, 0xFFFF, cfg.send_id)?;
        comm_server.set_iso15765_params(cfg.sep_time, cfg.block_size)?;
        
        let should_run = Arc::new(AtomicBool::new(true));
        let should_run_t = should_run.clone();

        let last_error = Arc::new(RwLock::new(None));
        let last_error_t = last_error.clone();

        let (channel_tx_sender, channel_tx_receiver): (
            Sender<(u8, Vec<u8>, bool)>,
            Receiver<(u8, Vec<u8>, bool)>,
        ) = mpsc::channel();
        let (channel_rx_sender, channel_rx_receiver): (
            Sender<ProtocolResult<Vec<u8>>>,
            Receiver<ProtocolResult<Vec<u8>>>,
        ) = mpsc::channel();

        let s_id = cfg.send_id;
        std::thread::spawn(move || {
            println!("OBD2 server start!");
            while should_run_t.load(Ordering::Relaxed) {
                if let Ok(data) = channel_tx_receiver.try_recv() {
                    let res = Self::run_command_iso_tp(
                        comm_server.as_ref(),
                        s_id,
                        data.0,
                        &data.1,
                        data.2,
                    );
                    if channel_rx_sender.send(res).is_err() {
                        *last_error_t.write().unwrap() =
                            Some(ProtocolError::CustomError("Sender channel died".into()));
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_micros(100))
            }
            println!("OBD2 Server stop!");
            comm_server.close_iso15765_interface();
        });

        let mut server = ObdServer {
            should_run,
            cmd_mutex: Arc::new(Mutex::new(())),
            cmd_rx: Arc::new(channel_rx_receiver),
            cmd_tx: channel_tx_sender
        };
        std::thread::sleep(std::time::Duration::from_millis(100)); // Wait for diag server to start

        if let Some(r) = Service01::init(&server) {
            if let Ok(s) = r.get_chartable_pid(&server, 0x04) {
                println!("Engine load: {:?}", s);
            }
            if let Ok(s) = r.get_chartable_pid(&server, 0x05) {
                println!("Coolant temp: {:?}", s);
            }

            if let Ok(s) = r.get_chartable_pid(&server, 0x1C) {
                println!("OBD standard: {:?}", s);
            }
        }

        Ok(server)
    }

    fn exit_diag_session(&mut self) {
        self.should_run.store(false, Ordering::Relaxed);
    }

    fn run_command(&self, cmd: u8, args: &[u8]) -> super::ProtocolResult<Vec<u8>> {
        let _guard = self.cmd_mutex.lock().unwrap(); // We are allowed to send / receive!
        if self.cmd_tx.send((cmd, Vec::from(args), true)).is_err() {
            return Err(ProtocolError::CustomError("Channel Tx failed".into()));
        }
        let mut resp = self.cmd_rx.recv().unwrap()?;
        if resp[0] == 0x7F {
            Err(ProtocolError::ProtocolError(Box::new(ObdError::from_byte(0))))
        } else {
            Ok(resp)
        }
    }

    fn read_errors(&self) -> super::ProtocolResult<Vec<super::DTC>> {
        todo!()
    }

    fn is_in_diag_session(&self) -> bool {
        true // Always
    }

    fn get_last_error(&self) -> Option<String> {
        None
    }
}

impl Drop for ObdServer {
    fn drop(&mut self) {
        self.exit_diag_session();
    }
}