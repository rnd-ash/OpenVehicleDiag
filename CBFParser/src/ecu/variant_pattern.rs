use common::raf::Raf;
use crate::caesar::{CaesarError, creader};


#[derive(Debug, Copy, Clone)]
pub enum ECUType {
    /// ECU uses KWP2000 as its underlying diagnostic protocol
    KWP,
    /// ECU uses UDS as its underlying diagnostic protocol
    UDS,
    /// ¯\_(ツ)_/¯
    UNK,
}

impl Default for ECUType {
    fn default() -> Self {
        Self::UNK
    }
}

#[derive(Debug, Clone, Default)]
pub struct VariantPattern {
    unk_buffer_size: i32,
    unk_buffer: Vec<u8>,

    unk3: i32,
    unk4: i32,
    unk5: i32,

    // ECU Vendor name
    pub vendor_name: String,

    // ECU Vendor ID (If using KWP2000)
    kwp_vendor_id: i32,
    unk8: i32,
    unk9: i32,
    unk10: i32,

    unk11: u32,
    unk12: u32,
    unk13: u32,
    unk14: u32,
    unk15: u32,

    unk16: Vec<u8>,

    unk17: u32,
    unk18: u32,
    unk19: u32,
    unk20: u32,

    unk21: String,

    unk22: i32,
    unk23: i32,
    uds_vendor_id: i32,
    pattern_type: i32,

    /// ECU Vendor ID (If using UDS)
    variant_id: ECUType,

    base_addr: usize,
}

impl VariantPattern {
    pub fn new(reader: &mut Raf, base_addr: usize) -> std::result::Result<Self, CaesarError> {
        reader.seek(base_addr);

        let mut bitflags = reader.read_u32()?;

        let unk_buffer_size = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        println!("Processing Variant Pattern - Base address: 0x{:08X}", base_addr);
        let mut res = VariantPattern {
            unk_buffer_size,
            unk_buffer: creader::read_bitflag_dump(&mut bitflags, reader, unk_buffer_size as usize, base_addr)?,

            unk3: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk4: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            unk5: creader::read_primitive(&mut bitflags, reader, 0i32)?,
            vendor_name: creader::read_bitflag_string(&mut bitflags, reader, base_addr)?,

            kwp_vendor_id: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk8: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk9: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,
            unk10: creader::read_primitive(&mut bitflags, reader, 0i16)? as i32,

            base_addr,
            ..Default::default()
        };
        if let Err(e) = res.put_extras(reader, bitflags, base_addr) {
            eprintln!("Warning. Failed to add extra fields for Variant! ({:?})", e)
        }
        res.variant_id = if res.uds_vendor_id == 0 { ECUType::KWP } else { ECUType::UDS };
        Ok(res)
    }

    pub fn put_extras(&mut self, reader: &mut Raf, mut bitflags: u32, base_addr: usize) -> std::result::Result<(), CaesarError> {
        self.unk11 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk12 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk13 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk14 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk15 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk16 = creader::read_bitflag_dump(&mut bitflags, reader, 5, base_addr)?;
        self.unk17 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk18 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk19 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk20 = creader::read_primitive(&mut bitflags, reader, 0u8)? as u32;
        self.unk21 = creader::read_bitflag_string(&mut bitflags, reader, base_addr)?;
        self.unk22 = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        self.unk23 = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        self.uds_vendor_id = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        self.pattern_type = creader::read_primitive(&mut bitflags, reader, 0i32)?;
        Ok(())
    }

    pub fn get_vendor_id(&self) -> i32 {
        match self.variant_id {
            ECUType::KWP => self.kwp_vendor_id,
            ECUType::UDS => self.uds_vendor_id,
            ECUType::UNK => 0
        }
    }
}