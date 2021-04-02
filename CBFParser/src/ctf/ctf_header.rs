use std::{io::{Read, Write}};
use common::raf::Raf;
use crate::caesar::{CaesarError, creader};
use super::STUB_HEADER_SIZE;

#[derive(Debug, Clone, Default)]
pub struct CTFHeader {
    unk1: i32,
    qualifier: String,
    unk3: i32,
    unk4: i32,
    language_count: i32,
    language_table_offset: i32,
    unk7: String,
    base_addr: usize,
    pub languages: Vec<CTFLanguage>
}

impl CTFHeader {
    pub fn new(reader: &mut Raf, base_addr: usize, header_size: usize) -> std::result::Result<Self, CaesarError> {
        reader.seek(base_addr);
        let mut bit_flag = reader.read_u16()? as u32;
        let mut res = CTFHeader {
            base_addr,
            unk1: creader::read_primitive(&mut bit_flag, reader, 0i32)?,
            qualifier: creader::read_bitflag_string(&mut bit_flag, reader, base_addr)?,
            unk3: creader::read_primitive(&mut bit_flag, reader, 0i16)? as i32,
            unk4: creader::read_primitive(&mut bit_flag, reader, 0i32)?,
            language_count: creader::read_primitive(&mut bit_flag, reader, 0i32)?,
            language_table_offset: creader::read_primitive(&mut bit_flag, reader, 0i32)?,
            unk7:  creader::read_bitflag_string(&mut bit_flag, reader, base_addr)?,
            ..Default::default()
        };
        let lang_table_offset_relative = res.language_table_offset as usize + base_addr;
        for i in 0..res.language_count as usize {
            let table_offset = lang_table_offset_relative + (i*4);
            reader.seek(table_offset);
            let real_lang_entry_addr = reader.read_i32()? as usize + lang_table_offset_relative;
            res.languages.push(CTFLanguage::new(reader, real_lang_entry_addr, header_size)?)
        }
        Ok(res)
    }

    pub fn get_languages(&self, idx: usize) -> CTFLanguage {
        self.languages[idx].clone()
    }
}



#[derive(Debug, Clone, Default)]
pub struct CTFLanguage {
    qualifier: String,
    language_index: usize,
    string_pool_size: usize,
    offset_string_pool_base: usize,
    string_count: usize,
    strings: Vec<String>,
    base_addr: usize
}

impl CTFLanguage {
    pub fn new(reader: &mut Raf, base_addr: usize, header_size: usize) -> std::result::Result<Self, CaesarError> {
        reader.seek(base_addr);
        let mut language_bit_flags = reader.read_u16()? as u32;

        let mut res = CTFLanguage {
            base_addr,
            qualifier: creader::read_bitflag_string(&mut language_bit_flags, reader, base_addr)?,
            language_index: creader::read_primitive(&mut language_bit_flags, reader, 0i16)? as usize,
            string_pool_size: creader::read_primitive(&mut language_bit_flags, reader, 0i32)? as usize,
            offset_string_pool_base: creader::read_primitive(&mut language_bit_flags, reader, 0i32)? as usize,
            string_count: creader::read_primitive(&mut language_bit_flags, reader, 0i32)? as usize,
            ..Default::default()
        };
        res.load_strings(reader, header_size)?;
        Ok(res)
    }

    fn load_strings(&mut self, reader: &mut Raf, header_size: usize) -> std::result::Result<(), CaesarError> {
        let table_offset = header_size + STUB_HEADER_SIZE + 4;
        self.strings.clear();
        for i in 0..self.string_count {
            reader.seek(table_offset + (i*4));
            let string_offset = reader.read_i32()? as usize;
            reader.seek(table_offset + string_offset);
            self.strings.push(reader.read_cstr()?)
        }
        Ok(())
    }

    pub fn get_string(&self, idx: i32) -> Option<String> {
        if idx < 0 {
            return None
        }
        self.strings.get(idx as usize).cloned()
    }

    pub fn dump_language_table(&self, name: String) -> std::io::Result<()> {
        let mut file = std::fs::File::create(name)?;

        for (idx, string) in self.strings.iter().enumerate() {
            file.write_all(format!("{},\"\"\"\"{}\"\"\"\"\n", idx, string).as_bytes())?;
        }
        file.flush()?;
        Ok(())
    }

    pub fn load_language_table(&mut self, name: String) -> std::io::Result<()> {
        let mut file = std::fs::File::open(name)?;
        let mut r = String::new();
        file.read_to_string(&mut r)?;
        let list: Vec<(usize, String)> = r.split("\n").map(|s| {
            let split: Vec<String> = s.split(",\"").map(|u| u.to_string()).collect();
            if let Ok(idx) = split[0].parse::<usize>() {
                let string: String = split[1].replace("\"", "");
                Some((idx, string))
            } else {
                None
            }
        }).filter_map(|x|x).collect();

        for (idx, string) in list {
            self.strings[idx] = string
        }
        Ok(())
    }
}

