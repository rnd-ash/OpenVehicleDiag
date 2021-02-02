use std::{env, io::Write};
use std::fs::File;
use caesar::container;
use common::raf::Raf;
use common::schema::{OvdECU, variant::{ECUVariantDefinition, ECUVariantPattern}, diag::{dtc::ECUDTC, service::{Service, DataType, Parameter}}};
use ctf::cff_header;
use diag::preparation::InferredDataType;
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
            patterns: Vec::new(),
            errors: Vec::new(),
            services: Vec::new()
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

        variant.dtcs.iter().for_each(|e| {
            let error = ECUDTC {
                description: e.description.clone().unwrap_or("".into()),
                error_name: e.qualifier.clone(),
                summary: e.reference.clone().unwrap_or("".into()),
            };
            //if !error.error_name.is_empty() {
            ecu_variant.errors.push(error)
            //}
        });


        variant.services.iter().for_each(|s| {
            //println!("{:#?}",s);
            let mut service = Service {
                name: s.qualifier.clone(),
                description: s.name.clone().unwrap_or("".into()),
                input_type: DataType::None,
                payload: s.req_bytes.clone(),
                input_params: Vec::new(),
                output_params: Vec::new()
            };

            s.input_preparations.iter().for_each(|p| {
                service.input_params.push(
                    Parameter {
                        name: p.qualifier.clone(),
                        start_bit: p.bit_pos as usize,
                        length_bits: p.size_in_bits as usize,
                        dump: p.dump.clone(),
                        data_type: if_to_dt(&p.field_type)
                    }
                )
            });

            s.output_preparations.iter().for_each(|i| {
                i.iter().for_each(|p| {
                    service.output_params.push(
                        Parameter {
                            name: p.qualifier.clone(),
                            start_bit: p.bit_pos as usize,
                            length_bits: p.size_in_bits as usize,
                            dump: p.dump.clone(),
                            data_type: if_to_dt(&p.field_type)
                        }
                    )
                })
            });


            ecu_variant.services.push(service);
        });

        ecu.variants.push(ecu_variant);
    }
    let mut f = File::create(format!("{}.json", ecu.name)).expect("Cannot open output file");
    f.write_all(serde_json::to_string_pretty(&ecu).unwrap().as_bytes()).expect("Error writing output");
    println!("ECU decoding complete. Output file is {}.json. Have a nice day!", ecu.name)
}

fn if_to_dt(if_dt: &InferredDataType) -> DataType {
    match if_dt {
        InferredDataType::Unassigned => DataType::None,
        InferredDataType::Integer => DataType::Int,
        InferredDataType::NativeInfoPool => DataType::Enum,
        InferredDataType::NativePresentation => DataType::Enum,
        InferredDataType::UnhandledITT => DataType::None,
        InferredDataType::UnhandledSP17 => DataType::None,
        InferredDataType::Unhandled => DataType::None,
        InferredDataType::BitDump => DataType::Hex,
        InferredDataType::ExtendedBitDump => DataType::Hex
    }
}