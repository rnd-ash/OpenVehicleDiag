use std::{vec, borrow::Cow, marker::PhantomData, rc::Rc, cell::RefCell};
use common::{raf::Raf};
use creader::{CaesarPrimitive};
use interface_subtype::InterfaceSubType;
use crate::{caesar::{CaesarError, creader}, ctf::{STUB_HEADER_SIZE, cff_header::CFFHeader, ctf_header::CTFLanguage}, diag::{dtc::DTC, presentation::Presentation, service::Service}};
use self::{interface::ECUInterface, variant::ECUVariant};

pub mod variant_pattern;
pub mod variant;
pub mod interface;
pub mod interface_subtype;
pub mod com_param;


#[derive(Debug, Clone, Copy, Default)]
pub (crate) struct Block {
    block_offset: usize,
    entry_count: usize,
    entry_size: usize,
    block_size: usize,
}

impl Block {
    pub (crate) fn new(reader: &mut Raf, bitflags: &mut u32, relative_offset: usize) -> std::result::Result<Self, CaesarError> {
        Ok(Self {
            block_offset: creader::read_primitive(bitflags, reader,0i32)?.to_usize() + relative_offset,
            entry_count: creader::read_primitive(bitflags, reader, 0i32)?.to_usize(),
            entry_size: creader::read_primitive(bitflags, reader, 0i32)?.to_usize(),
            block_size: creader::read_primitive(bitflags, reader, 0i32)?.to_usize(),
        })
    }
}


#[derive(Debug, Clone, Default)]
pub struct ECU {
    pub qualifier: Cow<'static, str>,
    pub name: Option<Cow<'static, str>>,
    pub description: Option<Cow<'static, str>>,
    pub xml_version: Cow<'static, str>,
    pub iface_block_count: i32,
    pub iface_table_offset: i32,
    pub sub_iface_count: i32,
    pub sub_iface_offset: i32,
    pub class_name: Cow<'static, str>,
    pub unk7: Cow<'static, str>,
    pub unk8: Cow<'static, str>,

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

    pub (crate) presentations: Block,

    internal_presentations: Block,

    unk: Block,

    unk39: i32,
    base_addr: usize,

    pub interfaces: Vec<ECUInterface>,
    pub interface_sub_types: Vec<InterfaceSubType>,

    pub global_dtcs: Vec<Rc<RefCell<DTC>>>,
    pub global_presentations: Vec<Rc<Presentation>>,
    pub global_internal_presentations: Vec<Rc<Presentation>>,
    pub global_env_ctxs: Vec<Rc<Service>>,
    pub global_services: Vec<Rc<Service>>,
    pub variants: Vec<ECUVariant>,
}

impl ECU {
    pub (crate) fn new(reader: &mut Raf, lang: CTFLanguage, header: &CFFHeader, base_addr: usize) -> std::result::Result<Self, CaesarError> {
        
        let mut bitflags = reader.read_u32()?;
        let bitflags_ext = reader.read_u16()? as u32;

        let _unk_0 = reader.read_i32()?;

        println!("Processing ECU - Base address: 0x{:08X}", base_addr);
        let mut res = ECU {
            base_addr,
            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            xml_version: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            iface_block_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            iface_table_offset: creader::read_primitive(&mut bitflags, reader, 0i32)?,
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
        res.unk_block_offset = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        res.sgml_source = creader::read_primitive(&mut bitflags, reader, 0i16)? as i32;
        res.unk6_relative_offset = creader::read_primitive(&mut bitflags, reader, 0i32)? as i32;

        res.ecu_variant = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.diag_job = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.dtc = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;

        res.env = Block {
            block_offset: creader::read_primitive(&mut bitflags, reader, 0i32)? as usize + data_buffer_offset_relative,
            entry_count: creader::read_primitive(&mut bitflags, reader, 0i32)? as usize,
            entry_size: creader::read_primitive(&mut bitflags, reader, 0i32)? as usize,
            block_size: 0,
        };

        bitflags = bitflags_ext;
        res.env.block_size = creader::read_primitive(&mut bitflags, reader, 0i32)? as usize;

        res.vc_domain = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.presentations = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.internal_presentations = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.unk = Block::new(reader, &mut bitflags, data_buffer_offset_relative)?;
        res.unk39 = creader::read_primitive(&mut bitflags, reader, 0i32)?;


        let iface_table_address = base_addr + res.iface_table_offset as usize;

        for i in 0..res.iface_block_count as usize {
            reader.seek(iface_table_address + (i*4));
            let iface_block_count = reader.read_i32()? as usize;
            let ecu_iface_base_addr = iface_table_address + iface_block_count;
            res.interfaces.push(ECUInterface::new(reader, ecu_iface_base_addr, &lang)?)
        }

        let sub_type_table_address = base_addr + res.sub_iface_offset as usize;
        for i in 0..res.sub_iface_count as usize {
            reader.seek(sub_type_table_address + (i*4));
            let block_offset = reader.read_i32()? as usize;
            let sub_type_base_addr = sub_type_table_address + block_offset;
            res.interface_sub_types.push(InterfaceSubType::new(reader, sub_type_base_addr, i, &lang)?)
        }

        res.global_presentations = Self::create_presentations(reader, &lang, &res.presentations)?;
        res.global_internal_presentations = Self::create_presentations(reader, &lang, &res.internal_presentations)?;
        res.create_env(reader, &lang)?;
        res.create_diag_jobs(reader, &lang)?;

        // Create DTCs
        res.global_dtcs = Self::create_dtcs(reader, &lang, &res.dtc)?;

        // Create variants
        let tmp = &res.ecu_variant.clone();
        res.create_ecu_variants(reader, &lang, tmp)?;

        // Done building our ECU varients, we can destroy our working arrays
        res.global_env_ctxs.clear();
        res.global_services.clear();
        res.global_dtcs.clear();
        Ok(res)
    }

    pub (crate) fn read_pool(reader: &mut Raf, pool: &Block) -> std::result::Result<Vec<u8>, CaesarError> {
        reader.seek(pool.block_offset);
        reader.read_bytes(pool.entry_count * pool.entry_size).map_err(CaesarError::FileError)
    }

    fn create_dtcs(reader: &mut Raf, lang: &CTFLanguage, dtc_blk: &Block) -> std::result::Result<Vec<Rc<RefCell<DTC>>>, CaesarError> {
        let pool = Self::read_pool(reader, dtc_blk)?;
        let mut res = vec![Rc::default(); dtc_blk.entry_count];

        let mut tmp_reader = Raf::from_bytes(&pool, common::raf::RafByteOrder::LE);

        for i in 0..dtc_blk.entry_count {
            let offset = tmp_reader.read_i32()? as usize;
            let _size = tmp_reader.read_i32()?;
            let _crc = tmp_reader.read_i32()?;
            let dtc_base_address = offset + dtc_blk.block_offset;
            res[i] = Rc::new(RefCell::new(DTC::new(reader, dtc_base_address, i, lang)?));
        }
        Ok(res)
    } 

    fn create_presentations(reader: &mut Raf, lang: &CTFLanguage, pres_blk: &Block) -> std::result::Result<Vec<Rc<Presentation>>, CaesarError> {
        let pool = Self::read_pool(reader, pres_blk)?;
        let mut res = vec![Rc::default(); pres_blk.entry_count];
        let mut tmp_reader = Raf::from_bytes(&pool, common::raf::RafByteOrder::LE);

        for i in 0..pres_blk.entry_count {
            let offset = tmp_reader.read_i32()? as usize;
            let _size = tmp_reader.read_i32()?;
            let pres_base_address = offset + pres_blk.block_offset;
            res[i] = Rc::new(Presentation::new(reader, pres_base_address, i, lang)?)
        }
        Ok(res)
    }

    fn create_env(&mut self, reader: &mut Raf, lang: &CTFLanguage) -> std::result::Result<(), CaesarError> {
        let env_blk = &self.env;
        let pool = Self::read_pool(reader, env_blk)?;
        let mut res = vec![Rc::default(); env_blk.entry_count];
        let mut tmp_reader = Raf::from_bytes(&pool, common::raf::RafByteOrder::LE);

        for i in 0..env_blk.entry_count {
            let offset = tmp_reader.read_i32()? as usize;
            let _size = tmp_reader.read_i32()?;
            let env_base_address = offset + env_blk.block_offset;
        
            res[i] = Rc::new(Service::new(reader, env_base_address, i, lang, self)?)
        }
        self.global_env_ctxs = res;
        Ok(())
    }

    fn create_diag_jobs(&mut self, reader: &mut Raf, lang: &CTFLanguage) -> std::result::Result<(), CaesarError> {
        let diag_blk = &self.diag_job;
        let pool = Self::read_pool(reader, diag_blk)?;
        let mut res = vec![Rc::default(); diag_blk.entry_count];
        let mut tmp_reader = Raf::from_bytes(&pool, common::raf::RafByteOrder::LE);

        for i in 0..diag_blk.entry_count {
            let offset = tmp_reader.read_i32()? as usize;
            let _size = tmp_reader.read_i32()?;
            let _crc = tmp_reader.read_i32()?;
            let _config = tmp_reader.read_u16()?;

            let diag_job_base_address = offset + diag_blk.block_offset;
        
            res[i] = Rc::new(Service::new(reader, diag_job_base_address, i, lang, self)?)
        }
        self.global_services = res;
        Ok(())
    }

    fn create_ecu_variants(&mut self, reader: &mut Raf, lang: &CTFLanguage, var_blk: &Block) -> std::result::Result<(), CaesarError> {
        let pool = Self::read_pool(reader, var_blk)?;
        let mut res = vec![ECUVariant::default(); var_blk.entry_count];
        let mut tmp_reader = Raf::from_bytes(&pool, common::raf::RafByteOrder::LE);

        for i in 0..var_blk.entry_count {
            let offset = tmp_reader.read_i32()? as usize;
            let size = tmp_reader.read_i32()? as usize;
            let _config = tmp_reader.read_u16()?;

            let variant_base_address = offset + var_blk.block_offset;
        
            res[i] = ECUVariant::new(reader, self, lang, variant_base_address, size)?
        }
        self.variants = res;
        Ok(())
    }
}