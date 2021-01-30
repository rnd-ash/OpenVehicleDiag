use common::raf::Raf;
use creader::{read_bitflag_string, read_primitive};

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage, ecu::{ECU, com_param::ComParameter}};


#[derive(Debug, Clone, Default, Copy)]
struct cData {
    count: usize,
    offset: usize,
}

impl cData {
    pub fn new(reader: &mut Raf, bf: &mut u32) -> std::result::Result<Self, CaesarError> {
        Ok(Self {
            count: creader::read_primitive(bf, reader, 0u32)? as usize,
            offset: creader::read_primitive(bf, reader, 0u32)? as usize,
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
    qualifier: String,
    name: Option<String>,
    description: Option<String>,

    data_class_service_type: u16,
    data_class_service_type_shifted: i32,

    is_executable: bool,
    client_access_level: i32,
    security_access_level: i32,

    t_com_param: cData,
    q: cData,
    r: cData,

    input_ref_name: String,

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

    pub com_params: Vec<ComParameter>
}

impl Service {
    pub fn new(reader: &mut Raf, base_addr: usize, pool_idx: usize, lang: &CTFLanguage, parent: &ECU) -> std::result::Result<Self, CaesarError> {
        println!("Processing Diagnostic service - Base address: {}", base_addr);

        reader.seek(base_addr);

        let mut bitflags = reader.read_u32()?;
        let mut bitflags_ext = reader.read_u32()?;



        let res = Self {
            base_addr,
            pool_idx,
            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),

            
            ..Default::default()
        };
        Ok(res)
    }
}