use std::io::Read;

use common::raf::Raf;

pub mod bmw_comm;
pub mod decoder;
pub mod file_reader;
pub mod fsm;
pub mod machine;
pub mod sg_functions;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        decode_file(&args[1]);
    } else {
        todo!("Help not implemented");
    }
}

fn decode_file(path: &str) {
    let mut f = std::fs::File::open(path).expect("Cannot open input file");
    let mut buffer = vec![0; f.metadata().unwrap().len() as usize];
    f.read_exact(&mut buffer).expect("Error reading file");
    println!("Have {} bytes", buffer.len());
    let mut br = Raf::from_bytes(&buffer, common::raf::RafByteOrder::LE);
    file_reader::BmwFileReader::new(path, &mut br);
}
