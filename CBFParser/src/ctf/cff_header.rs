use std::vec;

use common::raf::Raf;
use creader::read_bitflag_string;

use crate::caesar::{self, CaesarError, creader};



#[derive(Debug, Clone, Default)]
pub struct CFFHeader {
    caesar_version: i32,
    gpd_version: i32,
    ecu_count: i32,
    ecu_offset: i32,
    ctf_offset: i32,
    string_pool_size: i32,
    dsc_offset: i32,
    dsc_count: i32,
    dsc_entry_size: i32,
    cbf_version_string: String,
    gpd_version_string: String,
    xml_string: String,

    cff_header_size: i32,
    base_addr: usize,

    dsc_block_offset: usize,
    dsc_block_size: i32,

    dsc_pool: Vec<u8>
}

impl CFFHeader {
    pub fn new(reader: &mut Raf) -> std::result::Result<CFFHeader, CaesarError> {
        reader.seek(super::STUB_HEADER_SIZE);

        let cff_header_size = reader.read_i32()?;
        let base_addr = reader.pos;
        let mut bitflags = reader.read_u16()? as u32;

        let header = CFFHeader {
            base_addr,
            cff_header_size,
            caesar_version: creader::read_primitive(&mut bitflags, reader, 0)?,
            gpd_version: creader::read_primitive(&mut bitflags, reader, 0)?,
            ecu_count: creader::read_primitive(&mut bitflags, reader, 0)?,
            ecu_offset: creader::read_primitive(&mut bitflags, reader, 0)?,
            ctf_offset: creader::read_primitive(&mut bitflags, reader, 0)?,
            string_pool_size: creader::read_primitive(&mut bitflags, reader, 0)?,
            dsc_offset: creader::read_primitive(&mut bitflags, reader, 0)?,
            dsc_count: creader::read_primitive(&mut bitflags, reader, 0)?,
            dsc_entry_size: creader::read_primitive(&mut bitflags, reader, 0)?,

            cbf_version_string: creader::read_bitflag_string(&mut bitflags, reader, 0)?,
            gpd_version_string: creader::read_bitflag_string(&mut bitflags, reader, 0)?,
            xml_string: creader::read_bitflag_string(&mut bitflags, reader, 0)?,

            ..Default::default()
        };

        Ok(header)
    }
}
