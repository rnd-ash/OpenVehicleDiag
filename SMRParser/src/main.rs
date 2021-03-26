use std::{env, path::{self, Path}};
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
mod SMRFile;
mod BlowFishTable;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: SMRParser <SMR-D File> <OUTPUT DIR>");
    }
    let f = File::open(args[1].clone()).expect("Cannot open file");
    if !Path::new(&args[2]).exists() {
        std::fs::create_dir(args[2].clone()).expect("Could not create output dir");   
    }
    let mut buf = BufReader::new(f);
    let mut smr = SMRFile::SMRD::fromFile(&mut buf);
    let mut smrd = smr.read_file();
    
    
    for f in &smrd.extract_zips() {
        let mut dst = File::create(format!("{}/{}", args[2].clone(), f.name)).expect("Could not create output file!");
        dst.write_all(&f.bytes).expect("Could not write output file");
    }
}
