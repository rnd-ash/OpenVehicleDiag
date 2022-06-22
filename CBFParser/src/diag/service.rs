use std::borrow::Cow;

use common::raf::Raf;
use crate::{caesar::{CaesarError, PoolTuple, creader}, ctf::ctf_header::CTFLanguage, ecu::{ECU, com_param::ComParameter}};
use super::preparation::Preparation;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[allow(dead_code)]
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

impl ServiceType {
    pub fn from_raw(x: u16) -> ServiceType {
        match x {
            5 => Self::Data,
            7 => Self::Download,
            10 => Self::DiagnosticFunction,
            19 => Self::DiagnosticJob,
            21 => Self::Session,
            22 => Self::StoredData,
            23 => Self::Routine,
            24 => Self::IoControl,
            26 | 27 => Self::Unknown,
            _ => {
                eprintln!("Unknown service type {:02X}", x);
                Self::Unknown
            }
        }
    }
}

impl Default for ServiceType {
    fn default() -> Self {
        Self::Unknown
    }
}


#[derive(Debug, Clone, Default)]
pub struct Service {
    pub qualifier: Cow<'static, str>,
    pub name: Option<Cow<'static, str>>,
    pub description: Option<Cow<'static, str>>,

    data_class_service_type: u16,
    pub data_class_service_type_shifted: i32,
    pub service_type: ServiceType,

    is_executable: bool,
    client_access_level: i32,
    security_access_level: i32,

    t_com_param: PoolTuple,
    q: PoolTuple,
    r: PoolTuple,

    pub input_ref_name: Cow<'static, str>,

    u_prep: PoolTuple,
    v: PoolTuple,
    request_bytes: PoolTuple,
    w_out_pres: PoolTuple,

    field50: u16,

    negative_response_name: Cow<'static, str>,
    unk_str3: Cow<'static, str>,
    unk_str4: Cow<'static, str>,

    p: PoolTuple,
    diag_service_code: PoolTuple,

    s: PoolTuple,

    x: PoolTuple,

    y: PoolTuple,

    z: PoolTuple,

    pub req_bytes: Vec<u8>,

    pub (crate) base_addr: usize,

    pub (crate) pool_idx: usize,

    pub com_params: Vec<ComParameter>,

    pub input_preparations: Vec<Preparation>,
    pub output_preparations: Vec<Preparation>
}

impl Service {
    pub fn new(reader: &mut Raf, base_addr: usize, pool_idx: usize, lang: & CTFLanguage, parent: &ECU) -> std::result::Result<Self, CaesarError> {
        //println!("Processing Diagnostic service - Base address: 0x{:08X}", base_addr);
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

            t_com_param: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,
            q: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,
            r: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,

            input_ref_name: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,

            u_prep: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,
            v: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,

            request_bytes: PoolTuple::new_default(reader, &mut bitflags, 0i16, 0i32)?,

            w_out_pres: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,

            field50: creader::read_primitive(&mut bitflags, reader, 0u16)?,

            negative_response_name: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            unk_str3: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            unk_str4: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,

            p: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,

            diag_service_code: PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?,

            s: PoolTuple::new_default(reader, &mut bitflags, 0i16, 0i32)?,
            ..Default::default()
        };
        res.service_type = ServiceType::from_raw(res.data_class_service_type);

        bitflags = bitflags_ext;

        res.x = PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?;
        res.y = PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?;
        res.z = PoolTuple::new_default(reader, &mut bitflags, 0i32, 0i32)?;

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

            res.input_preparations.push(Preparation::new(reader, lang, prep_base_addr + prep_entry_offset, prep_entry_bit_pos, prep_entry_mode, parent, res.get_byte_count(), &res.input_ref_name)?);
        }


        let out_pres_base_addr = base_addr + res.w_out_pres.offset;
        for i in 0..res.w_out_pres.count {

            reader.seek(out_pres_base_addr + (i*8));
            // TODO
            let result_pres_count = reader.read_i32()? as usize;
            let result_pres_offset = reader.read_i32()? as usize;

            let mut res_pres_vec = Vec::new();

            for i in 0..result_pres_count {
                let prep_base_addr = out_pres_base_addr + result_pres_offset;
                reader.seek(prep_base_addr + (i*10));

                let prep_entry_offset = reader.read_i32()? as usize;
                let prep_entry_bit_pos = reader.read_i32()? as usize;
                let prep_entry_mode = reader.read_u16()?;

                res_pres_vec.push(Preparation::new(reader, lang, prep_base_addr + prep_entry_offset, prep_entry_bit_pos, prep_entry_mode, parent, res.get_byte_count(), &res.input_ref_name)?);
            }
            res.output_preparations.extend_from_slice(&res_pres_vec);
        }

        let com_param_base_address = base_addr + res.t_com_param.offset;
        for i in 0..res.t_com_param.count {
            reader.seek(com_param_base_address + (i*4));
            let cp_offset = reader.read_i32()? as usize;
            let cp_entry_base_address = com_param_base_address + cp_offset;
            res.com_params.push(ComParameter::new(reader, cp_entry_base_address, &parent.interfaces)?)
        }


        //if &res.name.clone().unwrap_or_default() == "Active Diagnostic Information: Active Diagnostic Variant" {
        //    println!("{:#?}", res);
        //    panic!("")
        //}
        Ok(res)
    }

    pub (crate) fn get_byte_count(&self) -> usize {
        self.request_bytes.count
    }
    // For converting to param tyoe only!
}