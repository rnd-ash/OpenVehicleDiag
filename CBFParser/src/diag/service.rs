use std::fs::read_to_string;

use common::raf::Raf;
use creader::{CaesarPrimitive, read_bitflag_string, read_primitive};

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage, ecu::{ECU, com_param::ComParameter}};

use super::preparation::Preparation;


#[derive(Debug, Clone, Default, Copy)]
struct cData {
    count: usize,
    offset: usize,
}

impl cData {
    pub fn new<T: CaesarPrimitive, X: CaesarPrimitive>(reader: &mut Raf, bf: &mut u32, d1: T, d2: X) -> std::result::Result<Self, CaesarError> {
        Ok(Self {
            count: creader::read_primitive(bf, reader, d1)?.to_usize() as usize,
            offset: creader::read_primitive(bf, reader, d2)?.to_usize() as usize,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ServiceType {
    Data = 5,
    Download = 7,
    DiagnosticFunction = 10,
    DiagnosticJob = 19,
    Session = 21,
    StoredData = 22,
    Routine = 23,
    IoControl = 24,
    Unknown
}

impl Default for ServiceType {
    fn default() -> Self {
        Self::Unknown
    }
}


#[derive(Debug, Clone, Default)]
pub struct Service {
    pub qualifier: String,
    name: Option<String>,
    description: Option<String>,

    data_class_service_type: u16,
    pub data_class_service_type_shifted: i32,

    is_executable: bool,
    client_access_level: i32,
    security_access_level: i32,

    t_com_param: cData,
    q: cData,
    r: cData,

    pub input_ref_name: String,

    u_prep: cData,
    v: cData,
    request_bytes: cData,
    w_out_pres: cData,

    field50: u16,

    negative_response_name: String,
    unk_str3: String,
    unk_str4: String,

    p: cData,
    diag_service_code: cData,

    s: cData,

    x: cData,

    y: cData,

    z: cData,

    pub req_bytes: Vec<u8>,

    base_addr: usize,

    pool_idx: usize,

    pub com_params: Vec<ComParameter>,

    pub input_preparations: Vec<Preparation>,
    pub output_preparations: Vec<Preparation>
}

impl Service {
    pub fn new(reader: &mut Raf, base_addr: usize, pool_idx: usize, lang: &CTFLanguage, parent: &ECU) -> std::result::Result<Self, CaesarError> {
        println!("Processing Diagnostic service - Base address: 0x{:08X}", base_addr);

        reader.seek(base_addr);
        let mut bitflags = reader.read_u32()?;
        let bitflags_ext = reader.read_u32()?;



        let mut res = Self {
            base_addr,
            pool_idx,

            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,

            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),

            data_class_service_type: creader::read_primitive(&mut bitflags, reader, 0u16)?,

            is_executable: creader::read_primitive(&mut bitflags, reader, 0u16)? > 0,
            client_access_level: creader::read_primitive(&mut bitflags, reader, 0u16)? as i32,
            security_access_level: creader::read_primitive(&mut bitflags, reader, 0u16)? as i32,

            t_com_param: cData::new(reader, &mut bitflags, 0i32, 0i32)?,
            q: cData::new(reader, &mut bitflags, 0i32, 0i32)?,
            r: cData::new(reader, &mut bitflags, 0i32, 0i32)?,

            input_ref_name: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,

            u_prep: cData::new(reader, &mut bitflags, 0i32, 0i32)?,
            v: cData::new(reader, &mut bitflags, 0i32, 0i32)?,

            request_bytes: cData::new(reader, &mut bitflags, 0i16, 0i32)?,

            w_out_pres: cData::new(reader, &mut bitflags, 0i32, 0i32)?,

            field50: creader::read_primitive(&mut bitflags, reader, 0u16)?,

            negative_response_name: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            unk_str3: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            unk_str4: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,

            p: cData::new(reader, &mut bitflags, 0i32, 0i32)?,

            diag_service_code: cData::new(reader, &mut bitflags, 0i32, 0i32)?,

            s: cData::new(reader, &mut bitflags, 0i16, 0i32)?,
            ..Default::default()
        };

        bitflags = bitflags_ext;

        res.x = cData::new(reader, &mut bitflags, 0i32, 0i32)?;
        res.y = cData::new(reader, &mut bitflags, 0i32, 0i32)?;
        res.z = cData::new(reader, &mut bitflags, 0i32, 0i32)?;

        res.data_class_service_type_shifted = 1 << (res.data_class_service_type - 1);

        if res.request_bytes.count > 0 {
            reader.seek(base_addr + res.request_bytes.offset);
            res.req_bytes = reader.read_bytes(res.request_bytes.count)?;
        }

        for i in 0..res.u_prep.count {
            let prep_base_addr = base_addr + res.u_prep.offset;
            reader.seek(prep_base_addr + (i*10));

            let prep_entry_offset = reader.read_i32()? as usize;
            let prep_entry_bit_pos = reader.read_i32()? as usize;
            let prep_entry_mode = reader.read_u16()?;

            res.input_preparations.push(Preparation::new(reader, lang, prep_base_addr + prep_entry_offset, prep_entry_bit_pos, prep_entry_mode, parent, &res)?);
        }
        Ok(res)
    }

    pub (crate) fn get_byte_count(&self) -> usize {
        return self.request_bytes.count;
    }

    pub (crate) fn get_p_count(&self) -> usize {
        return self.p.count;
    }
}