use std::{sync::{Arc, Mutex, RwLock, atomic::{AtomicBool, Ordering}, mpsc::{self, Receiver, Sender}}, vec};

use commapi::protocols::ProtocolError;

use crate::commapi::{self, comm_api::{ComServer, FilterType}, iface::{DynamicInterface, Interface, InterfaceConfig, InterfaceType, PayloadFlag}};

use self::{service01::Service01, service02::Service02, service03::Service03, service04::Service04, service05::Service05, service06::Service06, service07::Service07, service08::Service08, service09::Service09, service10::Service0A};

use super::{CommandError, DTC, DTCState, DiagCfg, ECUCommand, ProtocolResult, ProtocolServer, Selectable};

pub mod service01;
pub mod service02;
pub mod service03;
pub mod service04;
pub mod service05;
pub mod service06;
pub mod service07;
pub mod service08;
pub mod service09;
pub mod service10;
pub mod codes;

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

    fn from_byte(_b: u8) -> Self
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

#[derive(Debug, Clone)]
pub struct ObdServer {
    should_run: Arc<AtomicBool>,
    cmd_tx: Sender<(u8, Vec<u8>, bool)>,
    cmd_rx: Arc<Receiver<ProtocolResult<Vec<u8>>>>,
    cmd_mutex: Arc<Mutex<()>>,
    s01: Option<Service01>,
    s02: Option<Service02>,
    s03: Option<Service03>,
    s04: Option<Service04>,
    s05: Option<Service05>,
    s06: Option<Service06>,
    s07: Option<Service07>,
    s08: Option<Service08>,
    s09: Option<Service09>,
    s10: Option<Service0A>,
}

impl ObdServer {
    pub fn req_service09<T, F: Fn(&Service09) -> ProtocolResult<T>>(&self, func: F) -> ProtocolResult<T> {
        if let Some(s) = &self.s09 {
            func(s)
        } else {
            Err(ProtocolError::CustomError("Service not supported by ECU".into()))
        }
    }

    /// Return type
    /// .0 - SID supported?
    /// .1 - SID ID
    /// .2 - SID Name
    pub fn get_supported_services(&self) -> Vec<(bool, usize, String)> {
        let mut res = Vec::new();
        res.push((self.s01.is_some(), 0x01, "Sensor data".into()));
        res.push((self.s02.is_some(), 0x02, "Freeze frame data".into()));
        res.push((self.s03.is_some(), 0x03, "Read DTCs".into()));
        res.push((self.s04.is_some(), 0x04, "Clear DTCs".into()));
        res.push((self.s05.is_some(), 0x05, "O2 monitoring test results".into()));
        res.push((self.s06.is_some(), 0x06, "test results (other)".into()));
        res.push((self.s07.is_some(), 0x07, "Show pending DTCs".into()));
        res.push((self.s08.is_some(), 0x08, "Control operation".into()));
        res.push((self.s09.is_some(), 0x09, "Vehicle Info".into()));
        res.push((self.s10.is_some(), 0x0A, "Permanent DTCs".into()));
        return res;
    }


    // Used for services 03, 07 and 0A
    fn decode_dtc_resp(&self, bytes: &[u8], state: DTCState, res: &mut Vec<DTC>) {
        let num_dtcs = bytes[0];
        if num_dtcs == 0 {
            return
        }
        for idx in 0..num_dtcs as usize {
            let a = bytes[idx*2+1];
            let b = bytes[idx*2+2];
            let char = match (a & 0b11000000) >> 6 {
                0 => 'P',
                1 => 'C',
                2 => 'B',
                _ => 'U'
            };

            let second = match (a & 0b00001100) >> 4 {
                0 => '0',
                1 => '1',
                2 => '2',
                _ => '3',
            };
            // STD Hex representation
            let third = format!("{:1X}", a & 0b00001111);
            let fourth = format!("{:02X}", b);

            res.push(DTC {
                error: format!("{}{}{}{}", char, second, third, fourth),
                state: state,
                check_engine_on: state == DTCState::Stored || state == DTCState::Permanent,
                id: (a as u32) << 8 | b as u32,
            })
        }
    }

    pub fn get_dtc_desc(dtc: &DTC) -> String {
        codes::get_dtc_desc(dtc)
    }
}

impl ProtocolServer for ObdServer {
    type Command = OBDCmd;

    type Error = ObdError;

    fn start_diag_session(
        comm_server: &Box<dyn ComServer>,
        interface_type: InterfaceType,
        interface_cfg: InterfaceConfig,
        tx_flags: Option<Vec<PayloadFlag>>,
        diag_cfg: DiagCfg,
    ) -> super::ProtocolResult<Self> {
        if interface_type != InterfaceType::IsoTp && interface_type != InterfaceType::Iso9141 {
            return Err(ProtocolError::CustomError("OBD-II Can only be executed over ISO-TP or ISO9141".into()))
        }

        let mut dyn_interface = DynamicInterface::new(comm_server, interface_type, &interface_cfg)?.clone_box();
        if interface_type == InterfaceType::IsoTp {
            dyn_interface.add_filter(FilterType::IsoTP{id: diag_cfg.recv_id, mask: 0xFFFF, fc: diag_cfg.send_id})?;
        } else {
            return Err(ProtocolError::CustomError("OBD-II over ISO9141 is a WIP".into()))
        }
      
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

        let s_id = diag_cfg.send_id;
        std::thread::spawn(move || {
            println!("OBD2 server start!");
            while should_run_t.load(Ordering::Relaxed) {
                if let Ok(data) = channel_tx_receiver.try_recv() {
                    let res = Self::run_command_resp(
                        &mut dyn_interface,
                        &tx_flags,
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
            let _res = dyn_interface.close();
        });

        let mut server = ObdServer {
            should_run,
            cmd_mutex: Arc::new(Mutex::new(())),
            cmd_rx: Arc::new(channel_rx_receiver),
            cmd_tx: channel_tx_sender,
            s01: None,
            s02: None,
            s03: None,
            s04: None,
            s05: None,
            s06: None,
            s07: None,
            s08: None,
            s09: None,
            s10: None,
        };
        std::thread::sleep(std::time::Duration::from_millis(100)); // Wait for diag server to start

        if let Some(r) = Service01::init(&server) {
            server.s01 = Some(r)
        }
        server.s03 = Some(service03::Service03);
        if let Some(r) = Service09::init(&server) {
            server.s09 = Some(r)
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
        let resp = self.cmd_rx.recv().unwrap()?;
        if resp[0] == 0x7F {
            Err(ProtocolError::ProtocolError(Box::new(ObdError::from_byte(0))))
        } else {
            Ok(resp)
        }
    }

    fn read_errors(&self) -> super::ProtocolResult<Vec<super::DTC>> {
        let mut res: Vec<DTC> = Vec::new();
        if let Ok(resp) = self.run_command(0x03, &[]) { //  Stored DTCs
            println!("S03: {:02X?}", resp);
            self.decode_dtc_resp(&resp[1..], DTCState::Stored, &mut res);
        }
        if let Ok(resp) = self.run_command(0x07, &[]) { // Pending DTCs
            println!("S07: {:02X?}", resp);
            self.decode_dtc_resp(&resp[1..], DTCState::Pending, &mut res);
        }
        if let Ok(resp) = self.run_command(0x0A, &[]) { // Permanent DTCs
            println!("S0A: {:02X?}", resp);
            self.decode_dtc_resp(&resp[1..], DTCState::Permanent, &mut res);
        }
        return Ok(res);
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