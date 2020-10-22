use std::env;
use std::fs::File;
use std::io::BufReader;
mod cxf;
extern crate xml;
mod log;
mod caesar;
use cxf::*;
use xml::reader::{EventReader, XmlEvent};

fn help(err: String) -> ! {
    println!("Error: {}", err);
    println!("Usage:");
    println!("cbf_parser <INPUT.CBF>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => read_file(&args[1]),
        _ => help(format!("Invalid number of args: {}", args.len() - 1)),
    }
    println!("Hello, world!");
}

fn indent(size: usize) -> String {
    const INDENT: &'static str = "    ";
    (0..size)
        .map(|_| INDENT)
        .fold(String::with_capacity(size * INDENT.len()), |r, s| r + s)
}

fn print_xml(s: &String) {
    let parser = EventReader::from_str(s);
    let mut depth = 0;
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                println!("{}<{}>", indent(depth), name);
                depth += 1;
            }
            Ok(XmlEvent::EndElement { name }) => {
                depth -= 1;
                println!("{}</{}>", indent(depth), name);
            }
            Ok(XmlEvent::Characters(s)) => {
                println!("{}{}", indent(depth), s);
            }
            Err(e) => {
                break;
            }
            _ => {}
        }
    }
}

fn read_file(path: &String) {
    let end_block: [u8; 2] = [0x00, 0x00];
    let end_block_single: [u8; 1] = [0x00];
    let f = File::open(path);
    if let Err(_) = f {
        help(format!("File {} does not exist", path));
    }
    let mut reader = BufReader::new(f.unwrap());
    // No clue on type
    let mut cxf = CxfFile::from_file(&mut reader).expect("Couldn't open CXF File!");
    match cxf.f_type {
        CxfType::CBF => println!("{:#?}", CbfFile::from_cxf(cxf).unwrap().read_header()),
        CxfType::CFF => println!("{:#?}", CbfFile::from_cxf(cxf).unwrap().read_header())
    }
}
