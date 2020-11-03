
use common::raf;
use crate::caesar::CReader;

pub const STUB_HEADER_SIZE: usize = 0x410;
pub const FILE_HEADER: &[u8] = "CBF-TRANSLATOR-VERSION:04.00".as_bytes();

pub struct BaseHeader{}
impl BaseHeader {
    pub fn read_header(header: &[u8]) {
        if &header[0..FILE_HEADER.len()] != FILE_HEADER {
            println!("Unknown CBF Version");
        } else {
            let cbf_header_id = header[0x401];
            if cbf_header_id != 3 {
                println!("Unrecogised magic: {:02X}", cbf_header_id);
            }
        }
    }
}


#[derive(Debug)]
pub struct CFFHeader {
    pub caser_version: i32,
    pub gpd_version: i32,
    pub ecu_count: i32,
    pub ecuOffsets: i32,
    pub ctf_header_rpos: i32,
    pub size_of_str_pool: i32,
    pub unk2_relative_offset: i32,
    pub instance_count2: i32,
    pub instance_count1: i32,
    pub cbf_version_string: String,
    pub gpd_version_string: String,
    pub diogenes_xml_string: String,
    pub cff_header_size: i32,
    pub base_address: i64,
}

impl CFFHeader {
    pub fn new(reader: &mut raf::Raf) -> Self {
        reader.seek(STUB_HEADER_SIZE);
        let cff_header_size = reader.read_i32().expect("Error reading header size");
        let base_address = reader.pos as i64;
        let mut bitflag = reader.read_u16().expect("Error reading bitflag") as u64;
        println!("Bitflag : {:08x}", bitflag);
        Self {
            caser_version: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            gpd_version: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            ecu_count: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            ecuOffsets: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            ctf_header_rpos: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            size_of_str_pool: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            unk2_relative_offset: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            instance_count2: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            instance_count1: CReader::read_bitflag_i32(&mut bitflag, reader, 0),
            cbf_version_string: CReader::read_bitflag_string(&mut bitflag, reader, 0).expect("CXF File has no CBF Header!"),
            gpd_version_string: CReader::read_bitflag_string(&mut bitflag, reader, 0).expect("CXF File has no GPD Header!"),
            diogenes_xml_string: CReader::read_bitflag_string(&mut bitflag, reader, 0).expect("CXF File has no XML!"),
            base_address,
            cff_header_size
        }
    }
}


#[derive(Debug)]
pub struct CTFLanguage {
    lang_name: Option<String>,
    lang_index: i32,
    str_pool_size: i32,
    str_pool_offset: i32,
    str_count: i32,
    strings: Vec<String>,
}

impl CTFLanguage {
    pub fn new(reader: &mut raf::Raf, base_addr: i64, header: &CFFHeader) -> Self {
        
        reader.seek(base_addr as usize);
        
        let mut language_entry_bitflag = reader.read_u16().expect("Failed to read language entry bitflag") as u64;
        let lang_name = CReader::read_bitflag_string(&mut language_entry_bitflag, reader, 0);
        let lang_index = CReader::read_bitflag_i16(&mut language_entry_bitflag, reader, 0) as i32;
        let str_pool_size = CReader::read_bitflag_i32(&mut language_entry_bitflag, reader, 0);
        let str_pool_offset = CReader::read_bitflag_i32(&mut language_entry_bitflag, reader, 0);
        let str_count = CReader::read_bitflag_i32(&mut language_entry_bitflag, reader, 0);
        let strings = CTFLanguage::load_strings(reader, header, str_count as usize);

        CTFLanguage {
            lang_name,
            lang_index,
            str_pool_size,
            str_pool_offset,
            str_count,
            strings
        }
    }

    fn load_strings(reader: &mut raf::Raf, header: &CFFHeader, str_count: usize) -> Vec<String> {
        let str_table_offset = (header.cff_header_size + 0x410 + 4) as usize;

        (0..str_count)
            .map(|i| {
                reader.seek(str_table_offset + (i*4));
                let offset = reader.read_i32().expect("Error reading String offset") as usize;
                reader.seek(str_table_offset + offset);
                CReader::read_string(reader)
            })
            .collect()
    }
}


#[derive(Debug)]
pub struct CTFHeader {
    ctf_unk1: i32,
    ctf_name: String,
    ctf_unk3: i32,
    ctf_unk4: i32,
    ctf_lang_count: i32,
    ctf_lang_table_offset: i32,
    ctf_unk_str: Option<String>,
    ctf_langs: Vec<CTFLanguage>
}

impl CTFHeader {
    pub fn new(reader: &mut raf::Raf, base_addr: i64, header: CFFHeader) -> Self {
        let base_addr = base_addr;
        reader.seek(base_addr as usize);

        let mut bitflag = reader.read_u16().expect("Error reading CTF Bit flag") as u64;

        let ctf_unk1 = CReader::read_bitflag_i32(&mut bitflag, reader, 0);
        let ctf_name = CReader::read_bitflag_string(&mut bitflag, reader, 0).expect("CTF doesn't have a name");
        let ctf_unk3 = CReader::read_bitflag_i16(&mut bitflag, reader, 0) as i32;
        let ctf_unk4 = CReader::read_bitflag_i32(&mut bitflag, reader, 0);
        let ctf_lang_count = CReader::read_bitflag_i32(&mut bitflag, reader, 0);
        let ctf_lang_table_offset = CReader::read_bitflag_i32(&mut bitflag, reader, 0);
        let ctf_unk_str = CReader::read_bitflag_string(&mut bitflag, reader, 0);

        let ctf_lang_table_offset_relative = ctf_lang_table_offset as i64 + base_addr as i64;

        let ctf_langs = (0..ctf_lang_count as i64).map(|lang_entry| {
            let lang_table_offset_entry = ctf_lang_table_offset_relative + (lang_entry * 4);
            reader.seek(lang_table_offset_entry as usize);
            CTFLanguage::new(reader, lang_table_offset_entry, &header)
        })
        .collect();

        Self {
            ctf_unk1,
            ctf_name,
            ctf_unk3,
            ctf_unk4,
            ctf_lang_count,
            ctf_lang_table_offset,
            ctf_unk_str,
            ctf_langs
        }
    }
}