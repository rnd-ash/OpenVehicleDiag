use std::{borrow::Cow, rc::Rc};

use common::raf::Raf;
use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage, ecu::ECU};
use super::{presentation::Presentation, service::Service};

const INT_SIZE_MAP: [u8; 7] = [0x00, 0x01, 0x04, 0x08, 0x10, 0x20, 0x40];

#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum InferredDataType {
    Unassigned,
    Integer,
    NativeInfoPool,
    NativePresentation,
    UnhandledITT,
    UnhandledSP17,
    Unhandled,
    BitDump,
    ExtendedBitDump
}

impl Default for InferredDataType {
    fn default() -> Self {
        Self::Unassigned
    }
}


#[derive(Debug, Clone, Default)]
pub struct Preparation {
    pub qualifier: Cow<'static, str>,
    pub name: Option<Cow<'static, str>>,

    unk1: i32,
    unk2: i32,
    alternative_bit_width: i32,
    itt_offset: i32,
    info_pool_idx: usize,
    pres_pool_idx: usize,
    field_1e: i32,
    system_param: i32,
    dump_mode: i32,
    dump_size: i32,
    pub dump: Vec<u8>,
    pub field_type: InferredDataType,

    pub bit_pos: usize,
    mode_cfg: u16,
    pub size_in_bits: i32,
    pub presentation: Option<Rc<Presentation>>
}

impl Preparation {
    pub fn new(reader: &mut Raf, lang: &CTFLanguage, base_addr: usize, bit_pos: usize, mode_cfg: u16, parent_ecu: &ECU, byte_count: usize, parent_service_name: &str) -> std::result::Result<Self, CaesarError> {
        //println!("Processing Diagnostic preparation - Base address: 0x{:08X}", base_addr);

        reader.seek(base_addr);

        let mut bitflags = reader.read_u32()?;

            //qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            //name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            //description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),

        let mut res = Self {
            bit_pos,
            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            unk1: creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            unk2: creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            alternative_bit_width: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            itt_offset: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            info_pool_idx: creader::read_primitive(&mut bitflags, reader, 0i32)? as usize,
            pres_pool_idx: creader::read_primitive(&mut bitflags, reader, 0i32)? as usize,
            field_1e: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            system_param: creader::read_primitive(&mut bitflags, reader, -1i16)? as i32,
            dump_mode: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            dump_size: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            mode_cfg,
            ..Default::default()
        };
        res.dump = creader::read_bitflag_dump(&mut bitflags, reader, res.dump_size as usize, base_addr)?;
        res.size_in_bits = res.get_size_in_bits(parent_ecu, byte_count, parent_service_name)?;
        Ok(res)
    }

    fn get_size_in_bits(&mut self, parent_ecu: &ECU, byte_count: usize, parent_service_name: &str) -> std::result::Result<i32, CaesarError> {
        let mode_e = self.mode_cfg & 0xF000;
        let mode_h = self.mode_cfg & 0x0FF0; // Param type
        let mode_l = self.mode_cfg & 0x000F;
        let mut result_bit_size: i32 = 0;


        if self.mode_cfg & 0xF00 == 0x300 { // DlDiagServiceRetrievePreparation
            if mode_l > 6 {
                return Err(CaesarError::ProcessException("impl_type <= 6. This data type does not exist!".into()))
            }

            match mode_h {
                0x320 => {
                    result_bit_size = INT_SIZE_MAP[mode_l as usize] as i32;
                    self.field_type = InferredDataType::Integer;
                }
                0x330 => {
                    result_bit_size = self.alternative_bit_width;
                    self.field_type = InferredDataType::BitDump;
                }
                0x340 => {
                    self.field_type = InferredDataType::UnhandledITT;
                    eprintln!("Warning - mode_h 0x340 is not implemented! - Data will be missing")
                }
                _ => eprintln!("Warning - mode_h is unrecognized value? 0x{:04X}", mode_h)
            }
        } else if self.system_param == -1 {
            if mode_e == 0x8000 {
                self.field_type = InferredDataType::NativePresentation;
                let pres = parent_ecu.global_internal_presentations[self.pres_pool_idx].clone();
                result_bit_size = if pres.type_length_1a > 0 { pres.type_length_1a } else { pres.type_length_bytes_maybe };
                if pres.type_1c == 0 { // Presentation is in bytes, convert length to bits
                    result_bit_size *= 8;
                }
                self.presentation = Some(pres);
            } else if mode_e == 0x2000 {
                self.field_type = InferredDataType::NativePresentation;
                let pres = parent_ecu.global_presentations[self.pres_pool_idx].clone();
                result_bit_size = if pres.type_length_1a > 0 { pres.type_length_1a } else { pres.type_length_bytes_maybe };
                if pres.type_1c == 0 { // Presentation is in bytes, convert length to bits
                    result_bit_size *= 8;
                }
                self.presentation = Some(pres);
            } else {
                return Err(CaesarError::ProcessException(format!("Unknown system type for {}. mode_cfg: {:04X} mode_e: {:04X} mode_h: {:04X} mode_l: {:04X}", self.qualifier, self.mode_cfg, mode_e, mode_h, mode_l)))
            }
        } else if mode_h == 0x410 {
            let reduced_sys_param = self.system_param - 0x10;
            if reduced_sys_param == 0 {
                // LOWBYTE (&0xFF)
                result_bit_size =(((byte_count & 0xFF) - (self.bit_pos / 8)) * 8) as i32;
                self.field_type = InferredDataType::ExtendedBitDump;
            } else if reduced_sys_param == 17 {
                if let Some(referenced_service) = parent_ecu.global_services.iter().find(|x| x.qualifier == parent_service_name) {
                    let has_request_data = referenced_service.get_byte_count() > 0;
                    let mut internal_type = referenced_service.data_class_service_type_shifted;
                    if internal_type & 0xC > 0 && has_request_data {
                        internal_type = match internal_type & 4 > 0 {
                            true => 0x10000000,
                            false => 0x20000000
                        }
                    } 
                    if internal_type & 0x10000 != 0 {
                        // reference type is global variable
                        result_bit_size = byte_count as i32;
                        self.field_type = InferredDataType::UnhandledSP17;
                    } else {
                        self.field_type = InferredDataType::UnhandledSP17;
                        result_bit_size = byte_count as i32 * 8;
                    }
                } else {
                    eprintln!("Warning - 0x410 '{}' has no matching parent diag service", self.qualifier);
                }
            } else {
                return Err(CaesarError::ProcessException(format!("invalid system parameter for {}", self.qualifier)))
            }
        } else if mode_h == 0x420 {
            if mode_l > 6 {
                return Err(CaesarError::ProcessException(format!("impl type <= 6 (Doesn't exist) for {}", self.qualifier)))
            }
            self.field_type = InferredDataType::Integer;
            result_bit_size = INT_SIZE_MAP[mode_l as usize] as i32;
        } else if mode_h == 0x430 {
            result_bit_size = self.alternative_bit_width;
            self.field_type = InferredDataType::BitDump;
        } else {
            self.field_type = InferredDataType::Unhandled;
            return Err(CaesarError::ProcessException(format!("Unhandled param type {} for {}", mode_h, self.qualifier)))
        }
        Ok(result_bit_size)
    }
}