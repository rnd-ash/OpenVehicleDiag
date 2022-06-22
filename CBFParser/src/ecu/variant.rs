use std::{ops::{Deref, DerefMut}, vec, borrow::Cow, marker::PhantomData, rc::Rc, cell::RefCell};
use common::raf::Raf;
use crate::{caesar::{CaesarError, PoolTuple, creader}, ctf::ctf_header::CTFLanguage, diag::{dtc::DTC, service::Service}};
use super::{ECU, variant_pattern::{VariantPattern}, com_param::ComParameter};

#[derive(Debug, Copy, Clone, Default)]
struct DTCPoolBounds {
    actual_index: i32,
    xref_start: i32,
    xref_count: i32
}

impl DTCPoolBounds {
    fn new(reader: &mut Raf) -> std::result::Result<Self, CaesarError> {
        Ok(Self {
            actual_index: reader.read_i32()?,
            xref_start: reader.read_i32()?,
            xref_count: reader.read_i32()?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ECUVariant {
    base_addr: usize,
    pub qualifier: Cow<'static, str>,
    pub name: Option<Cow<'static, str>>,
    pub description: Option<Cow<'static, str>>,
    pub (crate) unk_str1: Cow<'static, str>,
    pub (crate) unk_str2: Cow<'static, str>,
    pub (crate) unk1: i32,
    matching_parent: PoolTuple,
    subsection_b: PoolTuple,
    com_params: PoolTuple,
    diag_service_code: PoolTuple,
    diag_services: PoolTuple,
    dtc: PoolTuple,
    environment_ctx: PoolTuple,
    xref: PoolTuple,
    vc_domain: PoolTuple,
    pub (crate) negative_response_name: Cow<'static, str>,
    pub (crate) unk_byte: i32,

    pub variant_patterns: Vec<VariantPattern>,
    pub services: Vec<Rc<Service>>,
    pub dtcs: Vec<Rc<RefCell<DTC>>>,
    pub (crate) xref_list: Vec<i32>,
}

impl ECUVariant {
    /// Creates a new ECU variant from data stored
    /// inside the CBF file
    pub fn new(reader: &mut Raf, parent_ecu: &mut ECU, lang: &CTFLanguage, base_addr: usize, block_size: usize) -> std::result::Result<Self, CaesarError> {
        reader.seek(base_addr);

        let mut tmp_reader = Raf::from_bytes(&reader.read_bytes(block_size)?, common::raf::RafByteOrder::LE);

        let mut bitflags = tmp_reader.read_u32()?;
        let _skip = tmp_reader.read_u32()?;

        let mut res = Self {
            base_addr,
            qualifier: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, &mut tmp_reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, &mut tmp_reader, -1i32)?),
            unk_str1: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,
            unk_str2: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,

            unk1: creader::read_primitive(&mut bitflags, &mut tmp_reader, 0i32)?,
            matching_parent: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            subsection_b: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            com_params: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            diag_service_code: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            diag_services: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            dtc: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            environment_ctx: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,
            xref: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,

            vc_domain: PoolTuple::new_int(&mut tmp_reader, &mut bitflags)?,

            negative_response_name: creader::read_bitflag_string(&mut bitflags, &mut tmp_reader, 0)?,
            unk_byte: creader::read_primitive(&mut bitflags, &mut tmp_reader, 0u8)? as i32,
            ..Default::default()
        };
        println!("Processing ECU Variant - Base address: 0x{:08X}. Name: {}", base_addr, res.qualifier);
        tmp_reader.seek(res.diag_services.offset);
        let mut diag_services_pool_offsets: Vec<i32> = Vec::new();
        for _ in 0..res.diag_services.count {
            diag_services_pool_offsets.push(tmp_reader.read_i32()?)
        }
        
        tmp_reader.seek(res.dtc.offset);
        let mut dtc_pool_bounds: Vec<DTCPoolBounds> = Vec::new();
        for _ in 0..res.dtc.count {
            dtc_pool_bounds.push(DTCPoolBounds::new(&mut tmp_reader)?);
        }

        tmp_reader.seek(res.environment_ctx.offset);
        // TODO process ENV pool
        let mut env_ctx_pool_offsets: Vec<i32> = (0..res.environment_ctx.count)
            .into_iter()
            .map(|_| tmp_reader.read_i32())
            .filter_map(|x| x.ok())
            .collect();


        res.create_com_params(reader, parent_ecu)?;
        res.create_diag_services(diag_services_pool_offsets, parent_ecu)?;
        res.create_variant_patterns(reader)?;
        res.create_dtcs(res.dtc.count, &mut dtc_pool_bounds, parent_ecu)?;
        res.create_xrefs(reader)?;
        res.create_env_ctxs(&mut env_ctx_pool_offsets, parent_ecu)?;
        Ok(res)
    }

    fn create_xrefs(&mut self, reader: &mut Raf) -> std::result::Result<(), CaesarError> {
        self.xref_list = vec![0; self.xref.count];
        reader.seek(self.base_addr + self.xref.offset);
        for x in 0..self.xref.count {
            self.xref_list[x] = reader.read_i32()?;
        }
        Ok(())
    }

    fn create_diag_services(&mut self, pool: Vec<i32>, parent_ecu: &ECU) -> std::result::Result<(), CaesarError> {
        let mut res = vec![Rc::default(); pool.len()];
        parent_ecu.global_services.iter().for_each(|d| {
            for (pos, idx) in pool.iter().enumerate() {
                if d.pool_idx == *idx as usize {
                    res[pos] = d.clone();
                    break
                }
            }
        });
        self.services = res;
        Ok(())
    }

    fn create_com_params(&self, reader: &mut Raf, parent: &mut ECU) -> std::result::Result<(), CaesarError> {
        let base_addr = self.base_addr + self.com_params.offset;
        reader.seek(base_addr);

        let mut idxs: Vec<usize> = Vec::new();
        for _ in 0..self.com_params.count {
            idxs.push(reader.read_i32()? as usize + base_addr);
        }
        for offset in &idxs {
            let param = ComParameter::new(reader, *offset, &parent.interfaces)?;
            let parent_idx = if param.parent_iface_idx > 0 {
                param.parent_iface_idx
            } else {
                param.sub_iface_idx
            } as usize;

            if parent_idx < parent.interface_sub_types.len() {
                parent.interface_sub_types[parent_idx].comm_params.push(param);
            }
        }
        Ok(())
    }

    fn create_variant_patterns(&mut self, reader: &mut Raf) -> std::result::Result<(), CaesarError> {
        let table_offset = self.base_addr + self.matching_parent.offset;
        reader.seek(table_offset);
        let mut res: Vec<VariantPattern> = Vec::new();

        for i in 0..self.matching_parent.count {
            reader.seek(table_offset + (i*4));
            let ptn_offset = reader.read_i32()? as usize;
            res.push(VariantPattern::new(reader, ptn_offset + table_offset)?)
        }
        self.variant_patterns = res;
        Ok(())
    }

    fn create_dtcs(&mut self, count: usize, pool: &mut Vec<DTCPoolBounds>, parent: &ECU) -> std::result::Result<(), CaesarError> {
        let mut res: Vec<Option<Rc<RefCell<DTC>>>> = vec![None; count];
        parent.global_dtcs.iter().for_each(|dtc| {
            for i in 0..count {
                if dtc.deref().borrow().pool_idx == pool[i].actual_index as usize {
                    let mut d = dtc;
                    d.borrow_mut().xrefs_start = pool[i].xref_start;
                    d.borrow_mut().xrefs_count = pool[i].xref_count;
                    res[i] = Some(d.clone());
                }
            }
        });
        // Modify envs if global DTC
        let mut lowest = 0;
        let max = parent.global_dtcs.len();

        pool.sort_by(|x, y| x.actual_index.partial_cmp(&y.actual_index).unwrap());

        for i in 0..count {
            if res[i].is_none() {
                for idx in lowest..max {
                    if parent.global_dtcs[idx].deref().borrow().pool_idx == pool[i].actual_index as usize {
                        // Replace content with global!
                        let mut pdtc = &parent.global_dtcs[idx];
                        pdtc.borrow_mut().xrefs_start = pool[i].xref_start;
                        pdtc.borrow_mut().xrefs_count = pool[i].xref_count;
                        res[i] = Some(pdtc.clone());
                        lowest = idx;
                        break;
                    }
                }
            }
        }
        self.dtcs = res.into_iter().filter_map(|x| x).collect();
        Ok(())
    }

    fn create_env_ctxs(&mut self, offsets: &mut Vec<i32>, parent: &ECU) -> std::result::Result<(), CaesarError> {
        let mut ctxs: Vec<Rc<Service>> = vec![];
        
        for i in 0..offsets.len() {
            if i == offsets[i] as usize {
                ctxs.push(parent.global_env_ctxs[i].clone());
            }
        }

        for env in &parent.global_env_ctxs {
            for i in 0..offsets.len() {
                if env.pool_idx == offsets[i] as usize {
                    ctxs.push(env.clone())
                }
            }
        }
        // Now set them to DTCs!
        for d in self.dtcs.iter_mut() {
            let mut dtc = d.borrow_mut();
            for idx in dtc.xrefs_start..(dtc.xrefs_start+dtc.xrefs_count) {
                for s in &ctxs {
                    let xref = self.xref_list[idx as usize] as usize;
                    if s.pool_idx == xref {
                        dtc.envs.push(s.clone());
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}