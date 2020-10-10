use std::env;
use std::fs::File;
use std::io::BufReader;
mod cbf;
extern crate xml;
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
    let reader = BufReader::new(f.unwrap());
    let mut cbf = cbf::CbfFile::from_buf_reader(reader);
    println!("CBF HEADER:");
    println!("{}", cbf.read_header().unwrap());
    cbf.read_until_str("ORIGINAL")
        .expect("Could not find Header block in CBF");
    //println!("XML 1!");
    let str = cbf.read_string().unwrap();
    //print_xml(&str);
    println!("CBF VERSION: {}", cbf.read_string().unwrap()); // SKIP Version 2??
    println!("GPD VERSION: {}", cbf.read_string().unwrap()); // SKIP Version
                                                             //println!("XML 2!");
    let str2 = cbf.read_string().unwrap();
    //print_xml(&str2);
    cbf.read_until_str("ORIGINAL")
        .expect("Could not find Header block in CBF");
    println!("STRINGS");
    let mut i = 0;
    while cbf.peek_next_bytes(2) != end_block {
        println!("{} - {}", i, cbf.read_string().unwrap());
        i += 1;
    }
}
