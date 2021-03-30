use common::raf::Raf;
use crate::caesar::{CaesarError, creader};

#[derive(Debug, Clone, Default)]
pub struct CFFHeader {
    pub caesar_version: i32,
    pub gpd_version: i32,
    pub ecu_count: i32,
    pub ecu_offset: i32,
    pub ctf_offset: i32,
    pub string_pool_size: i32,
    pub dsc_offset: i32,
    pub dsc_count: i32,
    pub dsc_entry_size: i32,
    pub cbf_version_string: String,
    pub gpd_version_string: String,
    pub xml_string: String,

    pub cff_header_size: i32,
    pub base_addr: usize,

    pub dsc_block_offset: usize,
    pub dsc_block_size: i32,

    pub dsc_pool: Vec<u8>
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

            cbf_version_string: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            gpd_version_string: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            xml_string: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            ..Default::default()
        };

        // TODO DSC Pool

        Ok(header)
    }
}

