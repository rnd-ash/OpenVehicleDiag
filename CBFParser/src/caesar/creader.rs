use common::raf::Raf;

use super::CaesarError;


pub fn read_bitflag_string(bit_flag: &mut u32, reader: &mut Raf, base_addr: usize) -> super::Result<String> {
    if check_and_advance_bitflag(bit_flag) {
        let string_offset = reader.read_i32()? as usize;
        let reader_pos = reader.pos;
        reader.seek(string_offset + base_addr);
        let res = read_string(reader).unwrap_or("".into());
        reader.seek(reader_pos);
        Ok(res)
    } else {
        Ok("".into())
    }
}

pub fn read_bitflag_dump(bit_flag: &mut u32, reader: &mut Raf, dump_size: usize, base_addr: usize) -> super::Result<Vec<u8>> {
    if check_and_advance_bitflag(bit_flag) {
        let dump_offset = reader.read_i32()? as usize;
        let reader_pos = reader.pos;
        reader.seek(dump_offset + base_addr);
        let res = reader.read_bytes(dump_size).unwrap_or([].into());
        reader.seek(reader_pos);
        Ok(res)
    } else {
        Ok(vec![])
    }
}

#[allow(dead_code)]
pub fn read_bitflag_dump_as_string(bit_flag: &mut u32, reader: &mut Raf, dump_size: usize, base_addr: usize) -> super::Result<String> {
    let data = read_bitflag_dump(bit_flag, reader, dump_size, base_addr)?;
    Ok(String::from_utf8(data).unwrap())
}


/// Checks if the bitflag is enabled, then shifts it to the right
fn check_and_advance_bitflag(bit_flag: &mut u32) -> bool {
    let is_set = (*bit_flag & 1) > 0;
    *bit_flag >>= 1;
    is_set
}


/// Reads a null terminating C String from the file,
/// returning it as UTF-8 encoded
fn read_string(reader: &mut Raf) -> super::Result<String> {
    reader.read_cstr().map_err(CaesarError::FileError)
}


pub fn read_primitive<T: CaesarPrimitive>(bit_flag: &mut u32, reader: &mut Raf, default: T) -> super::Result<T> {
    T::read_bitflag(bit_flag, reader, default)
}


pub trait CaesarPrimitive: Sized {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self>;
    fn to_usize(&self) -> usize;
}

impl CaesarPrimitive for f32 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_f32().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl CaesarPrimitive for i32 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_i32().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl CaesarPrimitive for u32 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_u32().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl CaesarPrimitive for i16 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_i16().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl CaesarPrimitive for u16 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_u16().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl CaesarPrimitive for i8 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_i8().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}

impl CaesarPrimitive for u8 {
    fn read_bitflag(bit_flag: &mut u32, reader: &mut Raf, default: Self) -> super::Result<Self> {
        if check_and_advance_bitflag(bit_flag) {
            Ok(reader.read_u8().unwrap_or(default))
        } else {
            Ok(default)
        }
    }

    fn to_usize(&self) -> usize {
        *self as usize
    }
}