use common::raf::Raf;
use creader::read_bitflag_string;

use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};

use super::pres_types::scale::Scale;

#[derive(Debug, Clone, Default)]
pub struct Presentation {
    pub (crate) qualifier: String,
    pub (crate) description: Option<String>,
    pub (crate) scale_table_offset: i32,
    pub (crate) scale_count: i32,
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
    pub (crate) display_unit: Option<String>,
    pub (crate) unk11: i32,
    pub (crate) unk12: i32,
    pub (crate) unk13: i32,
    pub (crate) unk14: i32,
    pub (crate) unk15: i32,
    pub (crate) description2: Option<String>,
    pub (crate) unk17: i32,
    pub (crate) unk18: i32,
    pub (crate) unk19: i32,
    pub (crate) type_length_1a: i32,
    pub (crate) unk1b: i32,
    pub (crate) type_1c: i32,
    pub (crate) unk1d: i32,
    pub (crate) enumtype_1e: i32,
    pub (crate) unk1f: i32,
    pub (crate) unk20: i32,
    pub (crate) type_length_bytes_maybe: i32,
    pub (crate) unk22: i32,
    pub (crate) unk23: i32,
    pub (crate) unk24: i32,
    pub (crate) unk25: i32,
    pub (crate) unk26: i32,
    pub (crate) base_addr: usize,
    pub (crate) presentation_idx: usize,
    pub (crate) scale_list: Vec<Scale>
}

impl Presentation {
    pub fn new(reader: &mut Raf, base_addr: usize, presentation_idx: usize, lang: &CTFLanguage) -> std::result::Result<Self, CaesarError> {
        println!("Processing Diagnostic presentation - Base address: 0x{:08X}", base_addr);
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
}


    // Functions below are just for generation generic JSON and nothing to do with CBF decompiling
    #[derive(Debug, Clone)]
pub (crate) enum DataTypeCBF {
    /// Represents a boolean value from the data presentation.
    /// each boolean value (true/false) can have an optional
    /// string attached to it
    Bool{pos_str: Option<String>, neg_str: Option<String>},

    /// Table values
    /// This is a list of values, where each value is mapped
    Table(Vec<(usize, String)>),

    /// Linear values
    /// This uses a conversion formula of y=mx+c to convert from input to output value
    Linear{m: f32, c: f32}
}

impl DataTypeCBF {
    pub fn create(x: Vec<Scale>) -> Option<Self> {
        if x.is_empty() {
            None // No scale list
        }
        // (x.len() == 2 && x[1].index == 0xFF) checks if there is an 'undefined'
        // value, if there is, ignore it as the underlying value is still linear
        // (Daimler use 0xFF or 0xFFFF to represent undefined values)
        else if x.len() == 1 || (x.len() == 2 && (x[1].index == 0xFF || x[1].index == 0xFFFF)) { // This is a linear value
            Some(Self::Linear{
                m: x[0].multiply_factor,
                c: x[0].add_const_offset,
            })
        } else if x.len() > 1 && (x[0].add_const_offset != 0f32 || x[0].multiply_factor != 0f32) {
            eprintln!("Skipping reserved values");
            Some(Self::Linear{
                m: x[0].multiply_factor,
                c: x[0].add_const_offset,
            })
        }

        // This is a boolean
        else if x.len() == 2 && (x[0].index == 0 && x[1].index == 1) || (x[0].index == 1 && x[1].index == 0) {
            // Should be illegal if the table has some m and c values
            if x[0].multiply_factor != 0f32 || x[1].multiply_factor != 0f32 || x[0].add_const_offset != 0f32 || x[1].add_const_offset != 0f32 {
                panic!("Bool with mm and sf?? -> {:#?}", x)
            }
            let negative = x.iter().find(|n| n.index == 0).unwrap();
            let positive = x.iter().find(|n| n.index == 1).unwrap();
            Some(Self::Bool{
                pos_str: positive.enum_description.clone(),
                neg_str: negative.enum_description.clone()
            })
        }

        // This is a table value list
        else {
            // Should be illegal if the table has some m and c values
            for scale in x.iter() {
                if scale.add_const_offset != 0f32 || scale.multiply_factor != 0f32 {
                    panic!("Table with mm and sf?? -> {:#?}", x)
                }
            }
            let mut res: Vec<(usize, String)> = Vec::new();
            x.iter().for_each(|t| {
                res.push((t.index as usize, t.enum_description.clone().expect(format!("Table entry has no value name?? {:#?}", t).as_str())))
            });
            Some(Self::Table(res))
        }
    }
}