use common::raf::Raf;
use creader::read_bitflag_string;

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};

use super::ECU;



#[derive(Debug, Copy, Clone, Default)]
struct cOffset(i32, i32);

impl cOffset {
    pub fn new(reader: &mut Raf, bf: &mut u32) -> std::result::Result<Self, CaesarError> {
        Ok(Self(
            creader::read_primitive(bf, reader, 0i32)?,
            creader::read_primitive(bf, reader, 0i32)?
        ))
    }
}

#[derive(Debug, Clone, Default)]
struct ECUVariant {
    qualifier: String,
    name: Option<String>,
    description: Option<String>,
    unk_str1: String,
    unk_str2: String,
    unk1: i32,
    matching_parent: cOffset,
    subsection_b: cOffset,
    com_params: cOffset,
    diag_service_code: cOffset,
    diag_services: cOffset,
    dtc: cOffset,
    environment_ctx: cOffset,
    xref: cOffset,
    vc_domain: cOffset,
    negative_response_name: String,
    unk_byte: i32,
}

impl ECUVariant {
    pub fn new(reader: &mut Raf, parent_ecu: &ECU, lang: &CTFLanguage, base_addr: usize, block_size: usize) -> std::result::Result<Self, CaesarError> {
        println!("Processing ECU Variant - Base address: 0x{:08X}", base_addr);
        reader.seek(base_addr);

        let mut tmp_reader = Raf::from_bytes(&reader.read_bytes(block_size)?, common::raf::RafByteOrder::LE);

        let mut bitflags = tmp_reader.read_u32()?;
        let _skip = tmp_reader.read_u32();

        let res = Self {
            qualifier: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, &mut tmp_reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, &mut tmp_reader, -1i32)?),
            unk_str1: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,
            unk_str2: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,

            unk1: creader::read_primitive(&mut bitflags, &mut tmp_reader, 0i32)?,
            matching_parent: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            subsection_b: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            com_params: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            diag_service_code: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            diag_services: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            dtc: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            environment_ctx: cOffset::new(&mut tmp_reader, &mut bitflags)?,
            xref: cOffset::new(&mut tmp_reader, &mut bitflags)?,

            vc_domain: cOffset::new(&mut tmp_reader, &mut bitflags)?,

            negative_response_name: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,
            unk_byte: creader::read_primitive(&mut bitflags, &mut tmp_reader, 0u8)? as i32,
        };
        Ok(res)
    }
}