use binary_reader::{BinaryReader, Endian};
use encoding_rs::ISO_8859_10;
use crate::cxf::*;

pub struct CReader{}

impl CReader {

    pub fn read_bitflag_string(bitflag: &mut u64, reader: &mut BinaryReader, virtual_base: i64) -> Option<String> {
        match Self::check_and_advance_bitflag(bitflag) {
            true => {
                let str_offset = reader.read_i32().expect("Error reading string offset") as usize;
                let pos = reader.pos;
                reader.jmp(str_offset + virtual_base as usize);
                let res = Self::read_string(reader);
                reader.jmp(pos);
                Some(res)
            },
            false => None
        }
    }


    pub fn read_bitflag_dump(bitflag: &mut u64, reader: &mut BinaryReader, dump_size: i32, virtual_base: i64) -> Option<Vec<u8>> {
        match Self::check_and_advance_bitflag(bitflag) {
            true => {
                let str_offset = reader.read_i32().expect("Error reading offset") as usize;
                let pos = reader.pos;
                reader.jmp(str_offset + virtual_base as usize);
                let res = Self::read_array(reader, dump_size as usize);
                reader.jmp(pos);
                Some(res)
            },
            false => None
        }
    }

    fn read_array(reader: &mut BinaryReader, size: usize) -> Vec<u8> {
        Vec::from(reader.read(size).unwrap())
    }


    pub fn read_bitflag_i8(bitflag: &mut u64, reader: &mut BinaryReader, default: i8) -> i8 {
        match Self::check_and_advance_bitflag(bitflag) {
            true => reader.read_i8().expect("Error reading i8"),
            false => default
        }
    }

    pub fn read_bitflag_u8(bitflag: &mut u64, reader: &mut BinaryReader, default: u8) -> u8 {
        match Self::check_and_advance_bitflag(bitflag) {
            true => reader.read_u8().expect("Error reading u8"),
            false => default
        }
    }

    pub fn read_bitflag_i16(bitflag: &mut u64, reader: &mut BinaryReader, default: i16) -> i16 {
        match Self::check_and_advance_bitflag(bitflag) {
            true => reader.read_i16().expect("Error reading i16"),
            false => default
        }
    }

    pub fn read_bitflag_u16(bitflag: &mut u64, reader: &mut BinaryReader, default: u16) -> u16 {
        match Self::check_and_advance_bitflag(bitflag) {
            true => reader.read_u16().expect("Error reading u16"),
            false => default
        }
    }

    pub fn read_bitflag_i32(bitflag: &mut u64, reader: &mut BinaryReader, default: i32) -> i32 {
        match Self::check_and_advance_bitflag(bitflag) {
            true => reader.read_i32().expect("Error reading i32"),
            false => default
        }
    }

    pub fn read_bitflag_u32(bitflag: &mut u64, reader: &mut BinaryReader, default: u32) -> u32 {
        match Self::check_and_advance_bitflag(bitflag) {
            true => reader.read_u32().expect("Error reading u32"),
            false => default
        }
    }


    /// Checks if the lowest bit is set within the input int,
    /// Then shifts the input u16 1 bit to the right
    fn check_and_advance_bitflag(bitflag: &mut u64) -> bool {
        let is_set = (*bitflag & 1) > 0;
        *bitflag >>= 1;
        return is_set;
    }


    pub fn read_string(reader: &mut BinaryReader) -> String {
        String::from(reader.read_cstr().as_str())
    }

}

pub struct CContainer{
    cff_header: CFFHeader,
    ctf_header: CTFHeader
}
impl CContainer {
    pub fn new(reader: &mut BinaryReader) -> Self {
        reader.jmp(0);
        let header = &reader.data[0..STUB_HEADER_SIZE];
        println!("{}", String::from_utf8_lossy(header));
        BaseHeader::read_header(header);
        reader.adv(STUB_HEADER_SIZE);

        let cff_header_size = reader.read_i32().expect("Cannot read CFF Header size");
        // Ignore header for now
        reader.adv(cff_header_size as usize);

        Self {
            cff_header: Self::read_cff(reader),
            ctf_header: Self::read_ctf(reader)
        }
    }

    fn read_ctf(reader: &mut BinaryReader) -> CTFHeader {
        panic!("Oops")
    }

    fn read_cff(reader: &mut BinaryReader) -> CFFHeader {
        let cff_header = CFFHeader::new(reader);
        if cff_header.caser_version < 400 {
            panic!("Unhanded caesar version: {}", cff_header.caser_version);
        }
        //let str_table_offset = cff_header.cff_header_size + 0x410 + 4;
        //let after_str_table_offset = str_table_offset + cff_header.size_of_str_pool;
        println!("{:#?}", cff_header);
        cff_header
    }
}


#[test]
fn test_advance_bitflag() {
    let mut bf: u64 = 2; // 0b0000_0010
    assert_eq!(CReader::check_and_advance_bitflag(&mut bf), false);
    assert_eq!(bf, 1);
    assert_eq!(CReader::check_and_advance_bitflag(&mut bf), true);
}