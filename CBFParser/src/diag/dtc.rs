use std::{borrow::Cow, rc::Rc};

use common::raf::Raf;
use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};

use super::service::Service;

#[derive(Debug, Clone, Default)]
pub struct DTC {
    pub qualifier: Cow<'static, str>,
    pub description: Option<Cow<'static, str>>,
    pub reference: Option<Cow<'static, str>>,

    pub (crate) xrefs_start: i32,
    pub (crate) xrefs_count: i32,
    pub base_addr: usize,

    pub pool_idx: usize,
    pub envs: Vec<Rc<Service>>
}

impl DTC {
    pub fn new(reader: &mut Raf, base_addr: usize, pool_idx: usize, lang: &CTFLanguage) -> std::result::Result<Self, CaesarError> {
        println!("Processing DTC - Base address: 0x{:08X}", base_addr);
        
        reader.seek(base_addr);
        let mut bitflags = reader.read_u16()? as u32;
        
        Ok(DTC {
            pool_idx,
            base_addr,
            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            reference: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            xrefs_start: -1, // ECU Variant will set these values
            xrefs_count: -1, // ECU Variant will set these values
            envs: Vec::new() // ECUVariant will process this!
        })
    }
}