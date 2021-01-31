use common::raf::Raf;

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};

#[derive(Debug, Clone, Default)]
pub struct DTC {
    pub qualifier: String,
    pub description: Option<String>,
    pub reference: Option<String>,

    pub xrefs_start: i32,
    pub xrefs_count: i32,
    pub base_addr: usize,

    pub pool_idx: usize,
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
            xrefs_start: -1,
            xrefs_count: -1,
        })
    }
}