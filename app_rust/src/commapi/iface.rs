use std::{borrow::BorrowMut, collections::HashMap, sync::{Arc, Mutex}};
use std::fmt::Debug;

use super::{comm_api::{Capability, ComServer, ComServerError, FilterType, CanFrame, ISO15765Data}};


pub type InterfaceResult<T> = std::result::Result<T, ComServerError>;

#[derive(Debug)]
#[allow(non_camel_case_types, dead_code)]
pub enum IFACE_CFG {
    BAUDRATE,
    SEND_ID,
    RECV_ID,
    EXT_CAN_ADDR,
    EXT_ISOTP_ADDR,
    PAD_FLOW_CONTROL,
    ISOTP_BS,
    ISOTP_ST_MIN,
}

impl ToString for IFACE_CFG {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
#[allow(non_camel_case_types)]
pub enum PayloadFlag {
    ISOTP_PAD_FRAME,
    ISOTP_EXT_ADDR
}

#[derive(Debug, Clone)]
pub struct InterfacePayload {
    pub id: u32,
    pub data: Vec<u8>,
    pub flags: Vec<PayloadFlag>
}

impl InterfacePayload {
    pub fn is_flag_set(&self, f: PayloadFlag) -> bool {
        self.flags.iter().find(|x| *x == &f).is_some()
    }

    pub fn new(id: u32, data: &[u8]) -> Self {
        Self {
            id,
            data: Vec::from(data),
            flags: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    params: HashMap<String, u32>
}

impl InterfaceConfig {
    pub fn new() -> Self {
        Self {
            params: HashMap::new()
        }
    }

    pub fn add_param(&mut self, param_name: IFACE_CFG, param_value: u32) {
        self.params.insert(param_name.to_string(), param_value);
    }

    pub fn get_param_or_default(&self, param_name: IFACE_CFG, default: u32) -> u32 {
        *self.params.get(&param_name.to_string()).unwrap_or(&default)
    }

    pub fn get_param(&self, param_name: IFACE_CFG) -> InterfaceResult<u32> {
        match self.params.get(&param_name.to_string()) {
            Some(x) => Ok(*x),
            None => Err(ComServerError{ err_code: 999, err_desc: format!("Interface configuration has missing parameter {:?}", param_name)})
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
pub enum BufferType {
    TX,
    RX,
    BOTH
}

pub trait Interface: Send + Sync + Debug {
    fn setup(&mut self, cfg: &InterfaceConfig) -> InterfaceResult<()>;
    fn send_data(&mut self, data: &[InterfacePayload], timeout: u32) -> InterfaceResult<usize>;
    fn recv_data(&mut self, max: usize, timeout: u32) -> InterfaceResult<Vec<InterfacePayload>>;
    fn add_filter(&mut self, f: FilterType) -> InterfaceResult<u32>;
    fn rem_filter(&mut self, f_id: u32) -> InterfaceResult<()>;
    fn close(&mut self) -> InterfaceResult<()>;
    fn clear_buffer(&mut self, buffer_type: BufferType) -> InterfaceResult<()>;
    fn send_recv_data(&mut self, request: InterfacePayload, write_timeout: u32, read_timeout: u32) -> InterfaceResult<InterfacePayload> {
        self.clear_buffer(BufferType::RX)?;
        self.send_data(&[request], write_timeout)?;
        let res = self.recv_data(1, read_timeout)?;
        if res.is_empty() {
            return Err(ComServerError{err_code: 2, err_desc: "Timeout waiting".into()});
        } else {
            return Ok(res[0].clone())
        }
    }
    fn get_server(&self) -> Box<dyn ComServer>;
    fn clone_box(&self) -> Box<dyn Interface>;
}

#[derive(Debug, Clone)]
pub struct CanbusInterface {
    dev: Box<dyn ComServer>
}

impl CanbusInterface {
    pub fn new(dev: Box<dyn ComServer>) -> InterfaceResult<Box<dyn Interface>> {
        if dev.get_capabilities().support_can_fd() != Capability::Yes {
            Err(ComServerError { err_code: 1, err_desc: "Device does not support CAN".into() })
        } else {
            Ok(Box::new(CanbusInterface{dev : dev.clone_box()}))
        }
    }

    pub fn new_raw(dev: Box<dyn ComServer>) -> Self {
        CanbusInterface{dev : dev.clone_box()}
    }
}

impl Interface for CanbusInterface {

    fn clear_buffer(&mut self, buffer_type: BufferType) -> InterfaceResult<()> {
        match buffer_type {
            BufferType::TX => self.dev.clear_can_tx_buffer(),
            BufferType::RX => self.dev.clear_can_rx_buffer(),
            BufferType::BOTH => {
                self.dev.clear_can_tx_buffer()?;
                self.dev.clear_can_rx_buffer()
            }
        }
    }

    fn setup(&mut self, cfg: &InterfaceConfig) -> InterfaceResult<()> {
        self.dev.open_can_interface(
            cfg.get_param(IFACE_CFG::BAUDRATE)?, 
            cfg.get_param_or_default(IFACE_CFG::EXT_CAN_ADDR, 0) > 0
        )
    }

    fn send_data(&mut self, data: &[InterfacePayload], timeout: u32) -> InterfaceResult<usize> {
        let can_packets: Vec<CanFrame> = data.iter().map(|f|{
            CanFrame::new(f.id, &f.data)
        }).collect();
        self.dev.send_can_packets(&can_packets, timeout)
    }

    fn recv_data(&mut self, max: usize, timeout: u32) -> InterfaceResult<Vec<InterfacePayload>> {
        self.dev.read_can_packets(timeout, max).map(|v| {
            v.iter().map(|f| {
                InterfacePayload {
                    id: f.id,
                    data: Vec::from(f.get_data()),
                    flags: vec![]
                }
            }).collect()
        })
    }

    fn close(&mut self) -> InterfaceResult<()> {
        self.dev.close_can_interface()
    }

    fn add_filter(&mut self, f_type: FilterType) -> InterfaceResult<u32> {
        self.dev.add_can_filter(f_type)
    }

    fn rem_filter(&mut self, f_id: u32) -> InterfaceResult<()> {
        self.dev.rem_can_filter(f_id)
    }

    fn get_server(&self) -> Box<dyn ComServer> {
        self.dev.clone_box()
    }

    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(Self {
            dev: self.dev.clone()
        })
    }
}

#[derive(Debug, Clone)]
pub struct IsoTPInterface {
    dev: Box<dyn ComServer>
}

impl IsoTPInterface {
    pub fn new(dev: Box<dyn ComServer>) -> InterfaceResult<Box<dyn Interface>> {
        if dev.get_capabilities().supports_iso15765() != Capability::Yes {
            Err(ComServerError { err_code: 1, err_desc: "Device does not support IsoTP".into() })
        } else {
            Ok(Box::new(IsoTPInterface{dev : dev.clone_box()}))
        }
    }
}

impl Interface for IsoTPInterface {

    fn clear_buffer(&mut self, buffer_type: BufferType) -> InterfaceResult<()> {
        match buffer_type {
            BufferType::TX => self.dev.clear_iso15765_tx_buffer(),
            BufferType::RX => self.dev.clear_iso15765_rx_buffer(),
            BufferType::BOTH => {
                self.dev.clear_iso15765_tx_buffer()?;
                self.dev.clear_iso15765_rx_buffer()
            }
        }
    }

    fn setup(&mut self, cfg: &InterfaceConfig) -> InterfaceResult<()> {
        self.dev.open_iso15765_interface(
            cfg.get_param(IFACE_CFG::BAUDRATE)?, 
                cfg.get_param_or_default(IFACE_CFG::EXT_CAN_ADDR, 0) > 0, 
                cfg.get_param_or_default(IFACE_CFG::EXT_ISOTP_ADDR, 0) > 0
        )?;
        // Use default if not specified
        self.dev.set_iso15765_params(
            cfg.get_param_or_default(IFACE_CFG::ISOTP_ST_MIN, 20),
            cfg.get_param_or_default(IFACE_CFG::ISOTP_BS, 8)
        )
    }

    fn send_data(&mut self, data: &[InterfacePayload], timeout: u32) -> InterfaceResult<usize> {
        let isotp_data: Vec<ISO15765Data> = data.iter().map(|t|{
            ISO15765Data {
                id: t.id,
                data: t.data.clone(),
                pad_frame: t.is_flag_set(PayloadFlag::ISOTP_PAD_FRAME),
                ext_addressing: t.is_flag_set(PayloadFlag::ISOTP_EXT_ADDR),
            }
        }).collect();
        self.dev.send_iso15765_data(&isotp_data, timeout)
    }

    fn recv_data(&mut self, max: usize, timeout: u32) -> InterfaceResult<Vec<InterfacePayload>> {
        self.dev.read_iso15765_packets(timeout, max).map(|v|{
            v.iter().map(|f| {
                InterfacePayload {
                    id: f.id,
                    data: f.data.clone(),
                    flags: vec![],
                }
            }).collect()
        })
    }

    fn close(&mut self) -> InterfaceResult<()> {
        self.dev.close_iso15765_interface()
    }

    fn add_filter(&mut self, f: FilterType) -> InterfaceResult<u32> {
        self.dev.add_iso15765_filter(f)
    }

    fn rem_filter(&mut self, f_id: u32) -> InterfaceResult<()> {
        self.dev.rem_iso15765_filter(f_id)
    }

    fn get_server(&self) -> Box<dyn ComServer> {
        self.dev.clone_box()
    }

    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(Self {
            dev: self.dev.clone()
        })
    }
}
#[derive(Debug, Clone)]
pub struct Iso14230Interface {
    dev: Box<dyn ComServer>
}

impl Iso14230Interface {
    pub fn new(dev: Box<dyn ComServer>) -> InterfaceResult<Box<dyn Interface>> {
        if dev.get_capabilities().supports_iso14230() != Capability::Yes {
            Err(ComServerError { err_code: 1, err_desc: "Device does not support ISO14230".into() })
        } else {
            Ok(Box::new(Iso14230Interface{dev : dev.clone_box()}))
        }
    }
}

#[allow(unused_variables)]
impl Interface for Iso14230Interface {
    fn clear_buffer(&mut self, buffer_type: BufferType) -> InterfaceResult<()> {
        todo!()
    }

    fn setup(&mut self, cfg: &InterfaceConfig) -> InterfaceResult<()> {
        todo!()
    }

    fn send_data(&mut self, data: &[InterfacePayload], timeout: u32) -> InterfaceResult<usize> {
        todo!()
    }

    fn recv_data(&mut self, max: usize, timeout: u32) -> InterfaceResult<Vec<InterfacePayload>> {
        todo!()
    }

    fn add_filter(&mut self, f: FilterType) -> InterfaceResult<u32> {
        todo!()
    }

    fn rem_filter(&mut self, f_id: u32) -> InterfaceResult<()> {
        todo!()
    }

    fn close(&mut self) -> InterfaceResult<()> {
        todo!()
    }

    fn get_server(&self) -> Box<dyn ComServer> {
        self.dev.clone_box()
    }

    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(Self {
            dev: self.dev.clone()
        })
    }
}
#[derive(Debug, Clone)]
pub struct Iso9141Interface {
    dev: Box<dyn ComServer>
}

impl Iso9141Interface {
    pub fn new(dev: Box<dyn ComServer>) -> InterfaceResult<Box<dyn Interface>> {
        if dev.get_capabilities().supports_iso9141() != Capability::Yes {
            Err(ComServerError { err_code: 1, err_desc: "Device does not support ISO9141".into() })
        } else {
            Ok(Box::new(Iso9141Interface{dev : dev.clone_box()}))
        }
    }
}

#[allow(unused_variables)]
impl Interface for Iso9141Interface {
    fn clear_buffer(&mut self, buffer_type: BufferType) -> InterfaceResult<()> {
        todo!()
    }

    fn setup(&mut self, cfg: &InterfaceConfig) -> InterfaceResult<()> {
        todo!()
    }

    fn send_data(&mut self, data: &[InterfacePayload], timeout: u32) -> InterfaceResult<usize> {
        todo!()
    }

    fn recv_data(&mut self, max: usize, timeout: u32) -> InterfaceResult<Vec<InterfacePayload>> {
        todo!()
    }

    fn close(&mut self) -> InterfaceResult<()> {
        todo!()
    }

    fn add_filter(&mut self, f: FilterType) -> InterfaceResult<u32> {
        todo!()
    }

    fn rem_filter(&mut self, f_id: u32) -> InterfaceResult<()> {
        todo!()
    }

    fn get_server(&self) -> Box<dyn ComServer> {
        self.dev.clone_box()
    }

    fn clone_box(&self) -> Box<dyn Interface> {
        Box::new(Self {
            dev: self.dev.clone()
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterfaceType {
    Can,
    IsoTp,
    Iso14230,
    Iso9141
}

#[derive(Debug, Clone)]
pub struct DynamicInterface {
    iface_type: Option<InterfaceType>,
    iface: Option<Arc<Mutex<Box<dyn Interface>>>>
}

impl DynamicInterface {
    pub fn new(server: &Box<dyn ComServer>, iface_type: InterfaceType, cfg: &InterfaceConfig) -> InterfaceResult<Self> {
        let mut iface = match iface_type {
            InterfaceType::Can => CanbusInterface::new(server.clone_box())?,
            InterfaceType::IsoTp => IsoTPInterface::new(server.clone_box())?,
            InterfaceType::Iso14230 => Iso14230Interface::new(server.clone_box())?,
            InterfaceType::Iso9141 => Iso9141Interface::new(server.clone_box())?
        };
        iface.setup(cfg)?;
        Ok(Self {
            iface_type: Some(iface_type),
            iface: Some(Arc::new(Mutex::new(iface)))
        })
    }

    pub fn blank() -> Self {
        Self {
            iface_type: None,
            iface: None
        }
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> &str {
        if let Some(s) = self.iface_type {
            match &s {
                InterfaceType::Can => "CAN",
                InterfaceType::IsoTp => "ISO15765 (ISO-TP)",
                InterfaceType::Iso14230 => "ISO14230 (KWP2000 over LIN)",
                InterfaceType::Iso9141 => "ISO9141 (OBD-II)"
            }
        } else {
            "Not configured"
        }
    }

    pub fn exec<R, F: Fn(&mut Box<dyn Interface>) -> InterfaceResult<R>>(&mut self, func: F) -> InterfaceResult<R> {
        match self.iface.as_mut() {
            Some(x) => func(x.lock().unwrap().borrow_mut()),
            None => Err(ComServerError{ err_code: 98, err_desc: "Dynamic interface not configured!".into() })
        }
    }
}

impl Interface for DynamicInterface {

    fn send_data(&mut self, data: &[InterfacePayload], timeout: u32) -> InterfaceResult<usize> {
        self.exec(|iface| iface.send_data(data, timeout))
    }

    fn recv_data(&mut self, max: usize, timeout: u32) -> InterfaceResult<Vec<InterfacePayload>> {
        self.exec(|iface| iface.recv_data(max, timeout))
    }

    fn add_filter(&mut self, f: FilterType) -> InterfaceResult<u32> {
        self.exec(|iface| iface.add_filter(f))
    }

    fn rem_filter(&mut self, f_id: u32) -> InterfaceResult<()> {
        self.exec(|iface| iface.rem_filter(f_id))
    }

    fn close(&mut self) -> InterfaceResult<()> {
        self.exec(|iface| iface.close())
    }

    fn clear_buffer(&mut self, buffer_type: BufferType) -> InterfaceResult<()> {
        self.exec(|iface| iface.clear_buffer(buffer_type))
    }

    fn send_recv_data(&mut self, request: InterfacePayload, write_timeout: u32, read_timeout: u32) -> InterfaceResult<InterfacePayload> {
        self.exec(|iface| iface.send_recv_data(request.clone(), write_timeout, read_timeout))
    }

    fn setup(&mut self, cfg: &InterfaceConfig) -> InterfaceResult<()> {
        self.exec(|iface| iface.setup(cfg))
    }

    fn get_server(&self) -> Box<dyn ComServer> {
        match &self.iface {
            Some(x) => x.lock().unwrap().get_server(),
            None => panic!("Illegal access of null interface")
        }
    }

    fn clone_box(&self) -> Box<dyn Interface> {
        match &self.iface {
            Some(x) => x.lock().unwrap().clone_box(),
            None => panic!("Illegal access of null interface")
        }
    }
}