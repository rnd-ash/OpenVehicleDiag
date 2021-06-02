use common::raf::Raf;


pub enum FileType {
    PRG,
    GRP
}

pub struct BmwFileReader{

}

impl BmwFileReader {
    pub fn new(path: &str, r: &mut Raf) -> Self {
        r.seek(0x10);
        let f_type = r.read_u32().unwrap();
        r.seek(0);
        println!("INIT {}, file type is {}", path, f_type);
        Self{}
    }
}