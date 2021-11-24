use std::{collections::HashMap, env, io::Write};
use std::fs::File;
use caesar::container;
use cbf_parser::diag::service::Service;
use diag::service::{ServiceType};
use common::{raf::Raf, schema::{Connection, diag::{DataFormat, StringEncoding, TableData}}};
use common::schema::{OvdECU, variant::{ECUVariantDefinition, ECUVariantPattern}, diag::{dtc::ECUDTC, service::{Parameter}}};
use diag::{preparation::InferredDataType};
use ecu::ECU;
use std::io::Read;

pub mod caesar;
pub mod ctf;
pub mod ecu;
pub mod diag;

type CService = common::schema::diag::service::Service;

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
            match container.read_ecus(reader) {
                Ok(_) => decode_ecu(&container.ecus[0]),
                Err(e) => {
                    eprintln!("Error decoding ECUS! {:?}", e)
                }
            }
        },
        Err(e) => {
            println!("ERROR PROCESSING {:?}", e)
        }
    }
}

fn decode_ecu(e: &ECU) {
    println!("Converting ECU {}", e.qualifier);

    let mut ecu = OvdECU {
        name: e.qualifier.clone(),
        description: e.name.clone().unwrap_or("".into()),
        variants: Vec::new(),
        connections: Vec::new()
    };

    let mut connections = Vec::new();
    for x in e.interface_sub_types.iter() {
        let connection = if x.qualifier.contains("CAN") { // Its CAN (ISOTP)
            Connection {
                baud: x.get_cp_by_name("CP_BAUDRATE").unwrap_or_default(),
                send_id:  x.get_cp_by_name("CP_REQUEST_CANIDENTIFIER").unwrap_or_default(),
                recv_id:  x.get_cp_by_name("CP_RESPONSE_CANIDENTIFIER").unwrap_or_default(),
                global_send_id: x.get_cp_by_name("CP_GLOBAL_REQUEST_CANIDENTIFIER"),
                connection_type: common::schema::ConType::ISOTP {
                    blocksize: 8, // Some reason MB always uses 8
                    st_min: x.get_cp_by_name("CP_STMIN_SUG").unwrap_or(20), // Seems default for MB
                    ext_isotp_addr: false, // MB never use extended ISO-TP addresing
                    ext_can_addr: x.get_cp_by_name("CP_REQUEST_CANIDENTIFIER").unwrap_or_default() > 0x7FF
                        || x.get_cp_by_name("CP_RESPONSE_CANIDENTIFIER").unwrap_or_default() > 0x7FF
                },
                server_type: if x.qualifier.contains("UDS") { // Interface type is in qualifier name for ISO-TP
                    common::schema::ServerType::UDS
                } else {
                     common::schema::ServerType::KWP2000
                }
            }
        } else {
            // Assume LIN
            println!("{:?}",x);
            Connection {
                baud: 10400, // Always for MB's LIN
                send_id: x.get_cp_by_name("CP_REQTARGETBYTE").expect("No LIN Request ID on interface!?"),
                recv_id: x.get_cp_by_name("CP_RESPONSEMASTER").expect("No LIN Response ID on interface!?"),
                global_send_id: x.get_cp_by_name("CP_TESTERPRESENTADDRESS"),
                connection_type: common::schema::ConType::LIN {
                    max_segment_size: x.get_cp_by_name("CP_SEGMENTSIZE").unwrap_or(254), // Default for ISO14230
                    wake_up_method: common::schema::LinWakeUpType::FiveBaudInit, // MB always uses this with KWP2000 LIN
                },
                server_type: common::schema::ServerType::KWP2000 // Always with LIN
            }
        };
        connections.push(connection);
    }
    ecu.connections = connections;
    for variant in e.variants.iter() {
        if variant.qualifier == e.qualifier {
            continue
        }

        let mut ecu_variant = ECUVariantDefinition {
            name: variant.qualifier.clone(),
            description: variant.name.clone().unwrap_or("".into()),
            patterns: Vec::new(),
            errors: Vec::new(),
            adjustments: Vec::new(),
            actuations: Vec::new(),
            functions: Vec::new(),
            downloads: Vec::new(),
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
            let mut error = ECUDTC {
                description: e.description.clone().unwrap_or("".into()),
                error_name: e.qualifier.clone(),
                summary: e.reference.clone().unwrap_or("".into()),
                envs: Vec::new()
            };

            for env in &e.envs {
                // Ok so envs only have 1 output param (ALWAYS!)
                // so we can copy the name and description to the output param
                let prep = &env.output_preparations[0];
                if let Some(pres) = &prep.presentation {
                    if let Some(data_fmt) = pres.create(prep) {
                        // Copy name and description from service
                        let param = Parameter {
                            name: env.name.clone().unwrap_or(prep.qualifier.clone()),
                            unit: pres.display_unit.clone().unwrap_or("".into()),
                            start_bit: prep.bit_pos,
                            length_bits: prep.size_in_bits as usize,
                            byte_order: common::schema::diag::service::ParamByteOrder::BigEndian,
                            data_format: data_fmt,
                            valid_bounds: None,
                        };
                        error.envs.push(param);
                    }
                }
            }

            //if !error.error_name.is_empty() {
            ecu_variant.errors.push(error)
            //}
        });

        variant.services.iter().for_each(|s| {
            let mut service = CService {
                name: s.qualifier.clone(),
                description: s.name.clone().unwrap_or("".into()),
                //input_type: DataType::None,
                payload: s.req_bytes.clone(),
                input_params: Vec::new(),
                output_params: Vec::new()
            };

            let mut tmp: Vec<Vec<u8>> = Vec::new();
            s.input_preparations.iter().for_each(|p| {
                if let Some(pres) = &p.presentation {
                    if let Some(data_fmt) = pres.create(p) {
                        let mut param = Parameter {
                            name: p.qualifier.clone(),
                            unit: pres.display_unit.clone().unwrap_or("".into()),
                            start_bit: p.bit_pos,
                            length_bits: p.size_in_bits as usize,
                            byte_order: common::schema::diag::service::ParamByteOrder::BigEndian,
                            data_format: data_fmt,
                            valid_bounds: None,

                        };
                        if let Some(name) = pres.description.clone() {
                            param.name = name;
                        }
                        tmp.push(p.dump.clone());
                        service.input_params.push(param);
                    }
                }
            });

            s.output_preparations.iter().for_each(|p| {
                if let Some(pres) = &p.presentation {
                    if let Some(data_fmt) = pres.create(p) {
                        let mut param = Parameter {
                            name: p.qualifier.clone(),
                            unit: pres.display_unit.clone().unwrap_or("".into()),
                            start_bit: p.bit_pos,
                            length_bits: p.size_in_bits as usize,
                            byte_order: common::schema::diag::service::ParamByteOrder::BigEndian,
                            data_format: data_fmt,
                            valid_bounds: None,

                        };
                        if let Some(name) = pres.description.clone() {
                            param.name = name;
                        }
                        service.output_params.push(param);
                    }
                }
                
            });

            // For CBF, it appears input params are repeated in the payload.
            // Delete them
            delete_input_params(&service.payload, &mut service.input_params, tmp);

            // Only add if we have a valid payload (Functions like {{INITIALIZATION}} are ignored)
            if !service.payload.is_empty() {
                match s.service_type {
                    ServiceType::Data | ServiceType::StoredData => ecu_variant.downloads.push(service),
                    ServiceType::DiagnosticFunction => ecu_variant.functions.push(service),
                    ServiceType::Routine => ecu_variant.functions.push(service),
                    _ => {
                        
                    }
                }
            }
        });
        println!("Data: {}, Diag Func: {}, Routine: {}", ecu_variant.downloads.len(), ecu_variant.functions.len(), ecu_variant.functions.len());

        // We need to cleanup the data functions. Seems MB has multiple functions that all use the same payload
        // Except output params differ
        //
        // OVD does this in bulk. So 1 function -> List all output params
        let unsorted = ecu_variant.downloads.clone();
        let mut map: HashMap<Vec<u8>,Vec<CService>> = HashMap::new();
        for s in &unsorted {
            if let Some(t) = map.get_mut(&s.payload) {
                t.push(s.clone())
            } else {
                map.insert(s.payload.clone(), vec![s.clone()]);
            }
        }
        ecu_variant.downloads.clear();

        // Now add our newly sorted data!
        for (_, mut service_list) in map {
            if service_list.len() == 1 {
                ecu_variant.downloads.push(service_list[0].clone()) // Easy
            } else {
                // Create a new service with all the output params!
                let mut root = service_list[0].clone();
                root.output_params[0].name = root.description.clone();
                root.name = format!("DT_{:02X}_{:02X}", root.payload[0], root.payload[1]);
                root.description = format!("Data download {:02X} {:02X}", root.payload[0], root.payload[1]);
                for s in service_list[1..].iter_mut() {
                    if s.output_params.len() == 1 {
                        let mut p = s.output_params[0].clone();
                        p.name = s.description.clone();
                        root.output_params.push(p);
                    } else {
                        ecu_variant.downloads.push(s.clone());
                    }
                }
                // Sort these
                root.output_params.sort_by(|x, y| x.start_bit.cmp(&y.start_bit));
                ecu_variant.downloads.push(root);
            }
        }
        ecu.variants.push(ecu_variant);
    }
    println!("SORTED");
    for v in &ecu.variants {
        println!("Data: {}, Diag Func: {}, Routine: {}", v.downloads.len(), v.functions.len(), v.functions.len());
    }
    println!("Writing to file");
    let mut f = File::create(format!("{}.json", ecu.name)).expect("Cannot open output file");
    f.write_all(serde_json::to_string_pretty(&ecu).unwrap().as_bytes()).expect("Error writing output");

    // Uncomment for SPLIT Json
    //for v in &ecu.variants {
    //    let new_ecu = OvdECU {
    //        name: format!("{}_{}", ecu.name, v.name),
    //        description: format!("{}. Variant {}", ecu.description, v.name),
    //        variants: vec![v.clone()],
    //        connections: ecu.connections.clone(),
    //    };
    //    let mut f = File::create(format!("{}.json", new_ecu.name)).expect("Cannot open output file");
    //    f.write_all(serde_json::to_string_pretty(&new_ecu).unwrap().as_bytes()).expect("Error writing output");
    //}

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