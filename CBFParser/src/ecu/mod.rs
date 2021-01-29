use std::{default, sync::Arc, vec};

use common::raf::Raf;
use creader::read_primitive;
use hyper::header::Basic;

use crate::{caesar::{CaesarError, container::Container, creader}, ctf::{STUB_HEADER_SIZE, cff_header::CFFHeader, ctf_header::CTFLanguage}};

pub mod variant_pattern;
pub mod variant;
pub mod interface;
pub mod interface_subtype;


#[derive(Debug, Clone, Copy, Default)]
struct Block {
    block_offset: i32,
    entry_count: i32,
    entry_size: i32,
    block_size: i32
}

impl Block {
    pub (crate) fn new(reader: &mut Raf, bitflags: &mut u32, relative_offset: usize) -> std::result::Result<Self, CaesarError> {
        Ok(Self {
            block_offset: creader::read_primitive(bitflags, reader, 0i32)? + relative_offset as i32,
            entry_count: creader::read_primitive(bitflags, reader, 0i32)?,
            entry_size: creader::read_primitive(bitflags, reader, 0i32)?,
            block_size: creader::read_primitive(bitflags, reader, 0i32)?
        })
    }
}


#[derive(Debug, Clone, Default)]
pub struct ECU {
    qualifier: String,
    name: Option<String>,
    description: Option<String>,
    xml_version: String,
    iface_block_count: i32,
    iface_table_count: i32,
    sub_iface_count: i32,
    sub_iface_offset: i32,
    class_name: String,
    unk7: String,
    unk8: String,

    ignition_required: bool,
    unk2: i32,

    unk_block_count: i32,
    unk_block_offset: i32,
    sgml_source: i32,
    unk6_relative_offset: i32,

    ecu_variant: Block,

    diag_job: Block,

    dtc: Block,

    env: Block,

    vc_domain: Block,

    presentations: Block,

    internal_presentations: Block,

    unk: Block,

    unk39: i32,
    base_addr: usize,
}

impl ECU {
    pub (crate) fn new(reader: &mut Raf, lang: &CTFLanguage, header: &CFFHeader, base_addr: usize, parent_container: Arc<Container>) -> std::result::Result<Self, CaesarError> {
        
        let mut bitflags = reader.read_u32()?;
        let mut bitflags_ext = reader.read_u16()? as u32;

        let unk_0 = reader.read_i32()?;

        println!("Processing ECU - Base address: {}", base_addr);
        let mut res = ECU {
            base_addr,
            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            xml_version: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            iface_block_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            iface_table_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            sub_iface_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            sub_iface_offset: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            class_name: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            unk7: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            unk8: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            ..Default::default()
        };

        let data_buffer_offset_relative = header.string_pool_size as usize + STUB_HEADER_SIZE + header.cff_header_size as usize + 4;

        res.ignition_required = creader::read_primitive(&mut bitflags, reader, 0i16)? > 0;
        res.unk2 = creader::read_primitive(&mut bitflags, reader, 0i16)? as i32;
        res.unk_block_count = creader::read_primitive(&mut bitflags, reader, 0i16)? as i32;
        res.sgml_source = creader::read_primitive(&mut bitflags, reader, 0i16)? as i32;
        res.unk6_relative_offset = creader::read_primitive(&mut bitflags, reader, 0i16)? as i32;

        res.ecu_variant = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.diag_job = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.dtc = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.env = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;

        bitflags = bitflags_ext;

        res.vc_domain = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.presentations = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.internal_presentations = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.unk = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.unk39 = creader::read_primitive(&mut bitflags, reader, 0i32)?;

        Ok(res)
    }
}