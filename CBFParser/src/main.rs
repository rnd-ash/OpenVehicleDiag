use std::env;
use std::fs::File;
use caesar::container;
use common::raf::Raf;
use common::schema::{OvdECU, variant::{ECUVariantDefinition, ECUVariantPattern}};
use ctf::cff_header;
use ecu::ECU;
use std::io::Read;

mod caesar;
mod ctf;
mod ecu;
mod diag;

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
}

fn read_file(path: &String) {
    if path.ends_with(".cff") {
        eprintln!("Cannot be used with CFF. Only CBF!");
        return;
    }
    let mut f = File::open(path).expect("Cannot open input file");
    let mut buffer = vec![0; f.metadata().unwrap().len() as usize];
    f.read_exact(&mut buffer).expect("Error reading file");
    println!("Have {} bytes", buffer.len());
    let mut br = Raf::from_bytes(&buffer, common::raf::RafByteOrder::LE);

    let container = container::Container::new(&mut br);
    match container {
        Ok(c) => decode_ecu(&c.ecus[0]),
        Err(e) => eprintln!("{:?}", e)
    }
}

fn decode_ecu(e: &ECU) {
    println!("Converting ECU {}", e.qualifier);

    let mut ecu = OvdECU {
        name: e.qualifier.clone(),
        description: e.name.clone().unwrap_or("".into()),
        variants: Vec::new()
    };

    for variant in e.variants.iter() {
        if variant.qualifier == e.qualifier {
            continue
        }

        let mut ecu_variant = ECUVariantDefinition {
            name: variant.qualifier.clone(),
            description: variant.name.clone().unwrap_or("".into()),
            patterns: Vec::new()
        };
        
        variant.variant_patterns.iter().for_each(|p| {
            ecu_variant.patterns.push(
                ECUVariantPattern {
                    vendor: p.vendor_name.clone(),
                    vendor_id: p.get_vendor_id()as u32,
                    hw_id: 0
                }
            );
        });

        ecu.variants.push(ecu_variant);
    }

    println!("{}", serde_json::to_string_pretty(&ecu).unwrap());
}