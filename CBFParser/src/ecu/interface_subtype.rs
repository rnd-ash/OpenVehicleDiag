use common::raf::Raf;
use creader::read_primitive;

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};

use super::com_param::ComParameter;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum ParamName {
    CP_BAUDRATE,
    CP_GLOBAL_REQUEST_CANIDENTIFIER,
    CP_FUNCTIONAL_REQUEST_CANIDENTIFIER,
    CP_REQUEST_CANIDENTIFIER,
    CP_RESPONSE_CANIDENTIFIER,
    CP_PARTNUMBERID,
    CP_PARTBLOCK,
    CP_HWVERSIONID,
    CP_SWVERSIONID,
    CP_SWVERSIONBLOCK,
    CP_SUPPLIERID,
    CP_SWSUPPLIERBLOCK,
    CP_ADDRESSMODE,
    CP_ADDRESSEXTENSION,
    CP_ROE_RESPONSE_CANIDENTIFIER,
    CP_USE_TIMING_RECEIVED_FROM_ECU,
    CP_STMIN_SUG,
    CP_BLOCKSIZE_SUG,
    CP_P2_TIMEOUT,
    CP_S3_TP_PHYS_TIMER,
    CP_S3_TP_FUNC_TIMER,
    CP_BR_SUG,
    CP_CAN_TRANSMIT,
    CP_BS_MAX,
    CP_CS_MAX,
    CPI_ROUTINECOUNTER,
    CP_REQREPCOUNT,
    // looks like outliers?
    CP_P2_EXT_TIMEOUT_7F_78,
    CP_P2_EXT_TIMEOUT_7F_21,
    CP_UNKNOWN // For default constructor
}

impl Default for ParamName {
    fn default() -> Self {
        Self::CP_UNKNOWN
    }
}


#[derive(Debug, Clone, Default)]
pub struct InterfaceSubType {
    qualifier: String,
    name: Option<String>,
    description: Option<String>,

    unk3: i32,
    unk4: i32,

    unk5: i32,
    unk6: i32,
    unk7: i32,

    unk8: i32,
    unk9: i32,
    unk10: i32,

    base_addr: usize,
    idx: usize,

    comm_params: Vec<ComParameter>
}

impl InterfaceSubType {
    pub fn new(reader: &mut Raf, base_addr: usize, idx: usize, lang: &CTFLanguage) -> std::result::Result<Self, CaesarError> {
        reader.seek(base_addr);

        let mut bitflags = reader.read_u32()?;
        let res = InterfaceSubType {
            base_addr,
            idx,

            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),

            unk3: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk4: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk5: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk6: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk7: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk8: creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            unk9: creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            unk10: creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            ..Default::default()
        };
        Ok(res)

    }
}

