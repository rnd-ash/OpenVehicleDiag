use std::env;
use std::fs::File;
use std::io::BufReader;
mod cbf;
extern crate xml;
mod log;
use log::Logger;
use xml::reader::{EventReader, XmlEvent};
extern crate byteorder;
use byteorder::{ByteOrder, LittleEndian};

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
    let mut cbf = cbf::CbfFile::fromFile(&mut reader).expect("Couldn't open CBF File!");
    
    
    let buf = cbf.readData(1040, 4).expect("CBF Metadata size could not be read");
    let bytes_to_read = LittleEndian::read_u32(&buf);
    let metadata = cbf.readData(1044, bytes_to_read as usize).expect("Could not read metadata portion");
    let temp_buffer = cbf.readData(0, 0x410).expect("Could not read CBF Header");
    let cbf_header = String::from_utf8_lossy(&temp_buffer);
    let metadata_str = String::from_utf8_lossy(&metadata);
    println!("{}", cbf_header);
    println!("{}", metadata_str);

    //println!("{:?}", header);
    //println!("{}", cbf.getOffset(0x07, 0x1D, 8));
}
