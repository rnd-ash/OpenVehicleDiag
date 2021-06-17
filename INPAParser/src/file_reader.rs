use common::raf::Raf;

use crate::machine::Machine;

pub enum FileType {
    PRG,
    GRP,
}

pub struct BmwFileReader {}

impl BmwFileReader {
    pub fn new(path: &str, r: &mut Raf) -> Self {
        r.seek(0x10);
        let f_type = r.read_u32().unwrap();
        r.seek(0);
        println!("INIT {}, file type is {}", path, f_type);
        //Self{}
        let mut m = Machine::new();
        m.load_file(r);

        for job_name in &m.get_job_names() {
            m.simulate_job(job_name);
        }

        Self {}
    }
}
