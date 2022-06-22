use std::borrow::{Borrow, Cow};

use common::{raf::Raf, schema::diag::{DataFormat, TableData}};
use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};
use super::{preparation::Preparation, pres_types::scale::Scale};

#[derive(Debug, Clone, Default)]
pub struct Presentation {
    pub qualifier: Cow<'static, str>,
    pub description: Option<Cow<'static, str>>,
    pub (crate) scale_table_offset: i32,
    pub scale_count: i32,
    pub (crate) unk5: i32,
    pub (crate) unk6: i32,
    pub (crate) unk7: i32,
    pub (crate) unk8: i32,
    pub (crate) unk9: i32,
    pub (crate) unka: i32,
    pub (crate) unkb: i32,
    pub (crate) unkc: i32,
    pub (crate) unkd: i32,
    pub (crate) unke: i32,
    pub (crate) unkf: i32,
    pub display_unit: Option<Cow<'static, str>>,
    pub (crate) unk11: i32,
    pub (crate) unk12: i32,
    pub (crate) unk13: i32,
    pub (crate) unk14: i32,
    pub (crate) unk15: i32,
    pub description2: Option<Cow<'static, str>>,
    pub (crate) unk17: i32,
    pub (crate) unk18: i32,
    pub (crate) unk19: i32,
    pub type_length_1a: i32,
    pub (crate) unk1b: i32,
    pub type_1c: i32,
    pub (crate) unk1d: i32,
    pub enumtype_1e: i32,
    pub (crate) unk1f: i32,
    pub (crate) unk20: i32,
    pub type_length_bytes_maybe: i32,
    pub (crate) unk22: i32,
    pub (crate) unk23: i32,
    pub (crate) unk24: i32,
    pub (crate) unk25: i32,
    pub (crate) unk26: i32,
    pub (crate) base_addr: usize,
    pub (crate) presentation_idx: usize,
    pub scale_list: Vec<Scale>
}

impl Presentation {
    pub fn new(reader: &mut Raf, base_addr: usize, presentation_idx: usize, lang: &CTFLanguage) -> std::result::Result<Self, CaesarError> {
        //println!("Processing Diagnostic presentation - Base address: 0x{:08X}", base_addr);
        reader.seek(base_addr);

        let mut bitflags = reader.read_u32()?;
        let bitflags_ext = reader.read_u16()? as u32;

        let mut res = Self {
            base_addr,
            presentation_idx,

            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            description: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            scale_table_offset: creader::read_primitive(&mut bitflags, reader, -1i32)?,
            scale_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,

            unk5: creader::read_primitive(&mut bitflags, reader, -1i32)?,
            unk6: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk7: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk8: creader::read_primitive(&mut bitflags, reader, 0i32)?,

            unk9: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unka: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unkb: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unkc: creader::read_primitive(&mut bitflags, reader, 0i32)?,

            unkd: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unke: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unkf: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            display_unit: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),

            unk11: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk12: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk13: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk14: creader::read_primitive(&mut bitflags, reader, -1i32)?,

            unk15: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            description2: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            unk17: creader::read_primitive(&mut bitflags, reader, -1i32)?,
            unk18: creader::read_primitive(&mut bitflags, reader, 0i32)?,

            unk19: creader::read_primitive(&mut bitflags, reader, -1i32)?,
            type_length_1a: creader::read_primitive(&mut bitflags, reader, -1i32)?,
            unk1b: creader::read_primitive(&mut bitflags, reader, -1i8)? as i32,
            type_1c: creader::read_primitive(&mut bitflags, reader, -1i8)? as i32,

            unk1d:  creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            enumtype_1e:  creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            unk1f:  creader::read_primitive(&mut bitflags, reader, 0i8)? as i32,
            unk20:  creader::read_primitive(&mut bitflags, reader, 0i32)?,
            ..Default::default()
        };

        bitflags = bitflags_ext;

        res.type_length_bytes_maybe = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        res.unk22 = creader::read_primitive(&mut bitflags, reader, -1i32)?;
        res.unk23 = creader::read_primitive(&mut bitflags, reader, 0i16)? as i32;
        res.unk24 = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        res.unk25 = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        res.unk26 = creader::read_primitive(&mut bitflags, reader, 0i32)?;

        if res.scale_count > 0 {
            let scale_table_base = base_addr + res.scale_table_offset as usize;
            for i in 0..res.scale_count as usize {
                reader.seek(scale_table_base + (i*4));
                let entry_offset = reader.read_i32()? as usize;
                res.scale_list.push(Scale::new(reader, entry_offset + scale_table_base, lang)?)
            }
        }
        Ok(res)
    }

    pub fn get_data_type(&self) -> i32 {
        let mut res: i32 = -1;
        if self.unk14 != -1 {
            return 17 // ASCII?
        }
        if self.scale_table_offset != -1 {
            return 20
        } else {
            if self.unk5 != -1 || self.unk17 != -1 || self.unk19 != -1 || self.unk22 != -1 {
                return 18
            }
            if self.unk1b != -1 {
                if self.unk1b == 6 {
                    return 17
                } else if self.unk1b == 7 {
                    return 22
                } else if self.unk1b == 8 || self.unk1b == 5 {
                    res = 6
                }
            } else {
                if self.type_length_1a == -1 || self.type_1c != -1 {
                    eprintln!("Type length and type must be valid")
                }
                if self.enumtype_1e == 1 || self.enumtype_1e == 2 {
                    res = 5;
                } else {
                    res = 2;
                }
            }
        }
        res
    }

    pub fn create(&self, prep: &Preparation) -> Option<DataFormat> {
        let is_enum = (self.enumtype_1e == 0 && self.type_1c == 1) || self.scale_list.len() > 1;
        if prep.size_in_bits == 1 || (is_enum && self.scale_list.len() == 2) {
            if self.scale_list.is_empty() { // If there is no enums in an enum value, assume true/false
                return Some(DataFormat::Bool { pos_name: None, neg_name: None })
            }
            if is_enum {
                return Some(DataFormat::Bool { pos_name: self.scale_list[1].enum_description.clone(), neg_name: self.scale_list[0].enum_description.clone() })
            } else {
                return Some(DataFormat::Identical) // Somehow a number with only 2 states??
            }
        }
        if is_enum && self.scale_count >= 1 {
            // Quick check to see if this is ACTUALLY a binary encoded string
            // CBF is evil. Binary encoded string = 256 entries of a scale table!
            // All binary stuff I've seen starts with 'b', so check for that as well

            let is_binary_str = self.scale_list.iter().map(|f| f.enum_description.clone().unwrap_or_default()).all(|x| x.starts_with('b'));
            if prep.size_in_bits <= 16 && self.scale_count == 2i32.pow(prep.size_in_bits as u32) && is_binary_str  {
                println!("Found Binary table with {} entries! {}", self.scale_count, self.qualifier);
                return Some(DataFormat::Binary)
            }

            let mut res: Vec<TableData> = Vec::new();
            for (_, s) in self.scale_list.iter().enumerate() {
                res.push(TableData {
                    name: s.enum_description.clone().unwrap_or("MISSING ENUM".into()),
                    start: s.enum_lower_bound as f32,
                    end: s.enum_upper_bound as f32,

                })
            }
            return Some(DataFormat::Table(res))
        }


        let d_type = self.get_data_type();
        if d_type == 6 {
            return Some(DataFormat::Identical)
        } else if d_type == 20 {
            if self.scale_list.is_empty() {
                eprintln!("Warning. Scale type {} has no scale list. Assuming identical", self.qualifier);
                return Some(DataFormat::Identical)
            } else {
                return Some(DataFormat::Linear { multiplier: self.scale_list[0].multiply_factor, offset: self.scale_list[0].add_const_offset })
            }
        } else if d_type == 18 {
            return Some(DataFormat::HexDump)
        } else if d_type == 17 {
            return Some(DataFormat::String(common::schema::diag::StringEncoding::Utf8))
        } else {
            return None
        }
    }
}