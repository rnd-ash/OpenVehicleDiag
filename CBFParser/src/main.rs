use std::{env, io::Write};
use std::fs::File;
use caesar::container;
use common::{raf::Raf, schema::diag::{DataFormat, StringEncoding, TableData}};
use common::schema::{OvdECU, variant::{ECUVariantDefinition, ECUVariantPattern}, diag::{dtc::ECUDTC, service::{Service, Parameter}}};
use diag::{preparation::InferredDataType, presentation::{DataTypeCBF}};
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
    println!("cbf_parser <INPUT.CBF> -dump_strings <STRINGS.csv>");
    println!("cbf_parser <INPUT.CBF> -load_strings <STRINGS.csv>");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 4 {
        match args[2].as_str() {
            "-dump_strings" => read_file(&args[1], Some(args[3].clone()), true),
            "-load_strings" => read_file(&args[1], Some(args[3].clone()), false),
            _ => help("String operation is not valid: {}".into())
        }
    } else if args.len() == 2 {
        read_file(&args[1], None, false)
    } else {
        help(format!("Invalid number of args: {}", args.len() - 1))
    }
}

fn read_file(path: &String, str_path: Option<String>, is_dump: bool) {
    if path.ends_with(".cff") {
        eprintln!("Cannot be used with CFF. Only CBF!");
        return;
    }
    let mut f = File::open(path).expect("Cannot open input file");
    let mut buffer = vec![0; f.metadata().unwrap().len() as usize];
    f.read_exact(&mut buffer).expect("Error reading file");
    println!("Have {} bytes", buffer.len());
    let mut br = Raf::from_bytes(&buffer, common::raf::RafByteOrder::LE);

    let c = container::Container::new(&mut br);


    match c {
        Ok((mut container, reader)) => {
            if let Some(p) = str_path {
                if is_dump {
                    return container.dump_strings(p)
                } else {
                    container.load_strings(p);
                }
            }
            if container.read_ecus(reader).is_ok() {
                decode_ecu(&container.ecus[0])
            }
        },
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
            if s.qualifier == "ACT_IO10_Idle_Speed" {
                println!("{:#?}", s)
            }
            let mut service = Service {
                name: s.qualifier.clone(),
                description: s.name.clone().unwrap_or("".into()),
                //input_type: DataType::None,
                payload: s.req_bytes.clone(),
                input_params: Vec::new(),
                output_params: Vec::new()
            };

            let mut tmp: Vec<Vec<u8>> = Vec::new();
            s.input_preparations.iter().for_each(|p| {
                let mut param = Parameter {
                    name: p.qualifier.clone(),
                    unit: "".into(),
                    start_bit: p.bit_pos, // Need to minus 8 as OVD starts are response start, CBF starts at message start
                    length_bits: p.size_in_bits as usize,
                    byte_order: common::schema::diag::service::ParamByteOrder::BigEndian, // Always
                    data_format: if_to_dt(&p.field_type, p.size_in_bits as usize),
                    limits: None
                };
                tmp.push(p.dump.clone());

                if param.name.contains("ASCII") {
                    param.data_format = DataFormat::String(StringEncoding::ASCII)
                }

                if let Some(pres) = &p.presentation {
                    param.unit = pres.display_unit.clone().unwrap_or("".into());
                    if let Some(name) = pres.description.clone() {
                        param.name = name;
                    }

                    if let Some(p) = DataTypeCBF::create(pres.scale_list.clone()) {
                        param.data_format = create_data_fmt(&p) // Calculate extended formatting type
                    }
                }
                service.input_params.push(param);
            });

            s.output_preparations.iter().for_each(|p| {
                    
                let mut param = Parameter {
                    name: p.qualifier.clone(),
                    unit: "".into(),
                    start_bit: p.bit_pos as usize,
                    length_bits: p.size_in_bits as usize,
                    byte_order: common::schema::diag::service::ParamByteOrder::BigEndian, // Always
                    data_format: if_to_dt(&p.field_type, p.size_in_bits as usize), // Get default (Basic) data type
                    limits: None
                };

                if param.name.contains("ASCII") {
                    param.data_format = DataFormat::String(StringEncoding::ASCII)
                }

                if let Some(pres) = &p.presentation {
                    param.unit = pres.display_unit.clone().unwrap_or("".into());
                    if let Some(name) = pres.description.clone() {
                        param.name = name;
                    }

                    if let Some(p) = DataTypeCBF::create(pres.scale_list.clone()) {
                        param.data_format = create_data_fmt(&p)
                    }
                }
                service.output_params.push(param);
            });

            // For CBF, it appears input params are repeated in the payload.
            // Delete them
            delete_input_params(&service.payload, &mut service.input_params, tmp);

            // Only add if we have a valid payload (Functions like {{INITIALIZATION}} are ignored)
            if !service.payload.is_empty() {
                ecu_variant.services.push(service);
            }
        });

        ecu.variants.push(ecu_variant);
    }
    let mut f = File::create(format!("{}.json", ecu.name)).expect("Cannot open output file");
    f.write_all(serde_json::to_string_pretty(&ecu).unwrap().as_bytes()).expect("Error writing output");
    println!("ECU decoding complete. Output file is {}.json. Have a nice day!", ecu.name)
}

fn delete_input_params(payload: &[u8], v: &mut Vec<Parameter>, dumps: Vec<Vec<u8>>) {
    let mut to_delete : Vec<usize> = Vec::new();

    for (pos, param) in v.iter().enumerate() {
        if param.length_bits == 8 {
            // Full byte, check
            let idx =  param.start_bit/8;

            if let Some(b) = payload.get(idx) {
                if let Some(x) = dumps[pos].get(0) {
                    if b == x {
                        to_delete.push(pos)
                    }
                }
            }
        }
    }

    for (pos, entry) in to_delete.iter().enumerate() {
        let real_idx = *entry - pos;
        v.remove(real_idx);
    }
}

fn if_to_dt(if_dt: &InferredDataType, length: usize) -> DataFormat {
    if length > 32 { // CBF is from a time of 16/32bit computing. No 64bit integers or floats exist
        return DataFormat::HexDump;
    }


    match if_dt {
        InferredDataType::Unassigned => DataFormat::HexDump,
        InferredDataType::Integer => DataFormat::Identical,
        InferredDataType::NativeInfoPool => DataFormat::Identical,
        InferredDataType::NativePresentation => DataFormat::Identical,
        InferredDataType::UnhandledITT => DataFormat::HexDump,
        InferredDataType::UnhandledSP17 => DataFormat::HexDump,
        InferredDataType::Unhandled => DataFormat::HexDump,
        InferredDataType::BitDump => DataFormat::HexDump,
        InferredDataType::ExtendedBitDump => DataFormat::HexDump,
        InferredDataType::String => DataFormat::String(StringEncoding::ASCII) // Always ASCII with CBF
    }
}

fn create_data_fmt(sf: &DataTypeCBF) -> DataFormat {

    match sf {
        DataTypeCBF::Bool { pos_str, neg_str } => {
            DataFormat::Bool {
                pos_name: pos_str.clone(),
                neg_name: neg_str.clone(),
            }
        },
        DataTypeCBF::Table(entries) => {

            let tables : Vec<TableData> = entries.iter().map(|(pos, x)| {
                TableData {
                    name: x.clone(),
                    start: *pos as f32,
                    end: *pos as f32
                }
            }).collect();
            DataFormat::Table(tables)
        },

        DataTypeCBF::Linear{m, c} => {
            DataFormat::Linear {
                multiplier: *m,
                offset: *c
            }
        }
    }
}