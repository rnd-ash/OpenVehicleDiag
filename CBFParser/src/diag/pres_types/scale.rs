use common::raf::Raf;
use creader::read_primitive;

use crate::caesar::{CaesarError, creader};


#[derive(Debug, Clone, Copy, Default)]
pub struct Scale {
    pub (crate) unk1: i32,
    pub (crate) unk2: i32,
    pub (crate) prep_lower_bound: i32,
    pub (crate) prep_upper_bound: i32,
    pub (crate) multiply_factor: f32,
    pub (crate) add_const_offset: f32,
    pub (crate) si_count: i32,
    pub (crate) offset_si: i32,
    pub (crate) us_count: i32,
    pub (crate) offset_us: i32,
    pub (crate) enum_description: i32,
    pub (crate) unkc: i32,
    pub (crate) base_addr: usize
}

impl Scale {
    pub fn new(reader: &mut Raf, base_addr: usize) -> std::result::Result<Self, CaesarError> {
        println!("Processing Scale data format - Base address: {}", base_addr);
        reader.seek(base_addr);

        let mut bitflags = reader.read_u16()? as u32;
        Ok(Self {
            base_addr,
            unk1: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk2: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            prep_lower_bound: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            prep_upper_bound: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            multiply_factor: creader::read_primitive(&mut bitflags, reader, 0f32)?,
            add_const_offset: creader::read_primitive(&mut bitflags, reader, 0f32)?,
            si_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            offset_si: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            us_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            offset_us: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            enum_description: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unkc: creader::read_primitive(&mut bitflags, reader, 0i32)?,
        })

    }
}