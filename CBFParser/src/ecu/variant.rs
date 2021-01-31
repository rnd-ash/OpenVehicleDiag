use common::raf::Raf;
use creader::{CaesarPrimitive, read_bitflag_string};

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage, diag::{dtc::{self, DTC}, service::Service}};

use super::{ECU, variant_pattern::{self, VariantPattern}};



#[derive(Debug, Copy, Clone, Default)]
pub (crate) struct cOffset{
    count: usize,
    offset: usize
}

impl cOffset {
    pub fn new(reader: &mut Raf, bf: &mut u32) -> std::result::Result<Self, CaesarError> {
        Ok(Self{
            count: creader::read_primitive(bf, reader, 0i32)?.to_usize(),
            offset: creader::read_primitive(bf, reader, 0i32)?.to_usize()
        })
    }
}

#[derive(Debug, Copy, Clone)]
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
    pub (crate) qualifier: String,
    pub (crate) name: Option<String>,
    pub (crate) description: Option<String>,
    pub (crate) unk_str1: String,
    pub (crate) unk_str2: String,
    pub (crate) unk1: i32,
    matching_parent: cOffset,
    subsection_b: cOffset,
    com_params: cOffset,
    diag_service_code: cOffset,
    diag_services: cOffset,
    dtc: cOffset,
    environment_ctx: cOffset,
    xref: cOffset,
    vc_domain: cOffset,
    pub (crate) negative_response_name: String,
    pub (crate) unk_byte: i32,

    pub (crate) variant_patterns: Vec<VariantPattern>,
    pub (crate) services: Vec<Service>,
    pub (crate) dtcs: Vec<DTC>,
}

impl ECUVariant {
    pub fn new(reader: &mut Raf, parent_ecu: &ECU, lang: &CTFLanguage, base_addr: usize, block_size: usize) -> std::result::Result<Self, CaesarError> {
        println!("Processing ECU Variant - Base address: 0x{:08X}", base_addr);
        reader.seek(base_addr);

        let mut tmp_reader = Raf::from_bytes(&reader.read_bytes(block_size)?, common::raf::RafByteOrder::LE);

        let mut bitflags = tmp_reader.read_u32()?;
        let _skip = tmp_reader.read_u32();

        let mut res = Self {
            base_addr,
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
            ..Default::default()
        };

        reader.seek(res.diag_services.offset);
        let diag_services_pool_offsets: Vec<i32> = (0..res.diag_services.count)
            .into_iter()
            .map(|_| reader.read_i32())
            .filter_map(|x| x.ok())
            .collect();

        reader.seek(res.dtc.offset);
        let dtc_pool_bounds: Vec<DTCPoolBounds> = (0..res.diag_services.count)
            .into_iter()
            .map(|_| DTCPoolBounds::new(reader))
            .filter_map(|x| x.ok())
            .collect();
        println!("{} {:?}", res.qualifier, dtc_pool_bounds);

        reader.seek(res.environment_ctx.offset);
        let env_ctx_pool_offsets: Vec<i32> = (0..res.environment_ctx.count)
            .into_iter()
            .map(|_| reader.read_i32())
            .filter_map(|x| x.ok())
            .collect();


        res.services = res.create_diag_services(diag_services_pool_offsets, parent_ecu)?;
        res.variant_patterns = res.create_variant_patterns(reader)?;
        res.dtcs = res.create_dtcs(reader, dtc_pool_bounds, parent_ecu)?;
        Ok(res)
    }

    fn create_diag_services(&self, pool: Vec<i32>, parent_ecu: &ECU) -> std::result::Result<Vec<Service>, CaesarError> {
        let mut res = vec![Service::default(); pool.len()];
        parent_ecu.global_diag_jobs.iter().for_each(|d| {
            for (pos, idx) in pool.iter().enumerate() {
                if d.pool_idx == *idx as usize {
                    res[pos] = d.clone()
                }
            }
        });
        Ok(res)
    }

    fn create_variant_patterns(&self, reader: &mut Raf) -> std::result::Result<Vec<VariantPattern>, CaesarError> {
        let table_offset = self.base_addr + self.matching_parent.offset;
        reader.seek(table_offset);
        let mut res: Vec<VariantPattern> = Vec::new();

        for i in 0..self.matching_parent.count {
            reader.seek(table_offset + (i*4));
            let ptn_offset = reader.read_i32()? as usize;
            println!("Pushing variant pattern for {}", self.qualifier);
            res.push(VariantPattern::new(reader, ptn_offset + table_offset)?)
        }

        Ok(res)
    }

    fn create_dtcs(&self, reader: &mut Raf, pool: Vec<DTCPoolBounds>, parent: &ECU) -> std::result::Result<Vec<DTC>, CaesarError> {
        let mut res = vec![DTC::default(); pool.len()];

        parent.global_dtcs.iter().for_each(|dtc| {
            for (pos, pool_bounds) in pool.iter().enumerate() {
                if dtc.pool_idx == pool_bounds.actual_index as usize {
                    res[pos] = dtc.clone();
                    res[pos].xrefs_start = pool_bounds.xref_start;
                    res[pos].xrefs_count = pool_bounds.xref_count;
                }
            }
        });
        Ok(res)
    }

}