use common::raf::Raf;
use crate::{caesar::{CaesarError, creader}, ctf::ctf_header::CTFLanguage};

#[derive(Debug, Clone, Default)]
pub struct ECUInterface {
    pub qualifier: String,
    name: Option<String>,
    desc: Option<String>,

    version_string: String,
    version: i32,
    com_param_count: i32,
    com_param_list_offset: i32,
    unk6: i32,
    pub com_params: Vec<String>,
    base_addr: usize
}

impl ECUInterface {
    pub fn new(reader: &mut Raf, base_addr: usize, lang: &CTFLanguage) -> std::result::Result<Self, CaesarError> {
        reader.seek(base_addr);
        println!("Processing ECU Interface - Base address: 0x{:08X}", base_addr);
        let mut bitflags = reader.read_u32()?;
        
        let mut res = ECUInterface {
            qualifier: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            name: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            desc: lang.get_string(creader::read_primitive(&mut bitflags, reader, -1i32)?),
            version_string: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,
            version: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            com_param_count: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            com_param_list_offset: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk6: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            base_addr,
            ..Default::default()
        };

        let com_param_file_offset = res.com_param_list_offset as usize + base_addr;

        for i in 0..res.com_param_count as usize {
            reader.seek(com_param_file_offset + (i*4));
            let iface_string_ptr = reader.read_i32()? as usize + com_param_file_offset;
            reader.seek(iface_string_ptr);
            let com_param = reader.read_cstr()?;
            res.com_params.push(com_param);
        }
        Ok(res)
    }
}