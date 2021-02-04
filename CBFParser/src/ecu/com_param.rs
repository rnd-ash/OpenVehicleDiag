use common::raf::Raf;
use crate::caesar::{CaesarError, creader};
use super::interface::ECUInterface;


#[derive(Debug, Clone, Default)]
pub struct ComParameter {
    param_idx: i32,
    parent_iface_idx: i32,
    sub_iface_idx: i32,
    unk5: i32,
    unk_ctf: i32,
    phrase: i32,
    dump_size: i32,
    dump: Vec<u8>,
    param_value: i32,
    param_name: String,

    base_addr: usize
}

impl ComParameter {
    pub fn new(reader: &mut Raf, base_addr: usize, parents: &[ECUInterface]) -> std::result::Result<Self, CaesarError> {
        println!("Processing COM Parameter - Base address: 0x{:08X}", base_addr);
        reader.seek(base_addr);
        let mut bitflags = reader.read_u16()? as u32;

        let mut res = ComParameter {
            param_idx: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            parent_iface_idx: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            sub_iface_idx: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk5: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk_ctf: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            phrase: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            dump_size: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            ..Default::default()
        };

        res.dump = creader::read_bitflag_dump(&mut bitflags, reader, res.dump_size as usize, base_addr)?;
        
        if res.dump_size == 4 {
            res.param_value = (res.dump[3] as i32) << 24 | (res.dump[2] as i32) << 16 | (res.dump[1] as i32) << 8 | res.dump[0] as i32;
        }

        let parent_iface = &parents[res.parent_iface_idx as usize];

        if res.param_idx as usize >= parent_iface.com_params.len() {
            res.param_name = "CP_MISSING_KEY".into();
            eprintln!("Warning. Communication parameter has no parent!. Value: {}, parent: {}", res.param_value, parent_iface.qualifier);
        } else {
            res.param_name = parent_iface.com_params[res.param_idx as usize].clone();
        }
        Ok(res)
    }
}