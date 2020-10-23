use std::env;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
mod SMRFile;
mod BlowFishTable;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: SMRParser <SMR-D File>");
    }
    let f = File::open(args[1].clone()).expect("Cannot open file");
    let mut buf = BufReader::new(f);
    let mut smr = SMRFile::SMRD::fromFile(&mut buf);
    let mut smrd = smr.read_file();
    smrd.extract_dlls();
    println!("Hello, world!");
}
