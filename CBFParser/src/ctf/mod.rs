pub mod cff_header;
pub mod ctf_header;


pub const STUB_HEADER_SIZE: usize = 0x410;
const FILE_HEADER: &'static [u8] = "CBF-TRANSLATOR-VERSION:04.00".as_bytes();

pub struct StubHeader;

impl StubHeader {
    pub fn read_header(header: &[u8]) {
        if !header[0..STUB_HEADER_SIZE].starts_with(FILE_HEADER) {
            eprintln!("WARNING. Unknown CBF version (Not 4.00.xx)")
        }
        let id = header[0x401];
        if id != 3 {
            eprintln!("WARNING. CBF Magic unrecognized ({})", id)
        }
    }
}