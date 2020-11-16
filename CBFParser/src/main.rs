use std::env;
use std::fs::File;
use common::raf::Raf;
mod cxf;
mod ecu;
mod diag;
mod structure;
extern crate xml;
mod log;
mod caesar;
use cxf::*;
use ecu::*;
use diag::*;
use xml::reader::{EventReader, XmlEvent};
use std::io::Read;

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
    let mut f = File::open(path).expect("Cannot open input file");
    let mut buffer = vec![0; f.metadata().unwrap().len() as usize];
    f.read(&mut buffer).expect("Error reading file");
    let mut br = Raf::from_bytes(&buffer, common::raf::RafByteOrder::LE);
    let container = caesar::CContainer::new(&mut br);
}
