 use crate::ecu::*;
 use crate::caesar::*;
 use crate::cxf::*;
 use common::raf;
 use crate::structure::*;
use serde::*;
const INT_SIZE_MAPPING: [u8; 7] = [0x00, 0x01, 0x04, 0x08, 0x10, 0x20, 0x40];


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DTC {

}

impl DTC {
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, base_addr: i64, pool_index: i32, parent_ecu: &ECU) {
        reader.seek(base_addr as usize);
        let mut bitflags = reader.read_u16().expect("Unable to read DiagService bitflags") as u64;
        let idx = CReader::read_bitflag_i32(&mut bitflags, reader, -1);
        let mut desc_txt = Vec::new();
        loop {
            let desc = CReader::read_bitflag_i32(&mut bitflags, reader, -1);
            if desc == -1 {
                break;
            } else {
                desc_txt.push(lang.get_string(desc));
            }
        }
        let name = reader.read_string(5);

        //let str_ref = reader.read_i32().unwrap();
        //let desc = lang.get_string(str_ref);
        //let name = reader.read_string(5);
        eprintln!("Name: {:?} IDX {} Desc: {:?}", name, idx, desc_txt);
    }
}

#[derive(Debug)]
pub enum ServiceType {
    Data = 5,
    Download = 7,
    DiagnosticFunction = 10,
    DiagnosticJob = 19,
    Session = 21,
    StoredData = 22,
    Routine = 23,
    IoControl = 24,
    VariantCodingWrite = 26,
    VariantCodingRead = 27
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagService{
    pub name: Option<String>,
    pub prep: Vec<DiagPreparation>,
    pub name_ctf: Option<String>,
    pub desc_ctf: Option<String>,
    pub is_executable: bool,
    pub client_access_level: u16,
    pub security_access_level: u16,
    pub req_bytes_count: i32,
    pub req_bytes_offset: i32,
    pub input_ref_name: Option<String>,
    pub dataclass_servicetype_shifted: i32,
    pub p_count: i32,
    pub pool_index: i32,
    pub t_comparam_count: i32,
    pub t_comparam_offset: i32,
    pub q_count: i32,
    pub q_offset: i32,
    pub r_count: i32,
    pub r_offset: i32,
    pub req_bytes: Vec<u8>,
    pub v_count: i32,
    pub v_offset: i32,
    pub w_outpres_count: i32,
    pub w_outpres_offset: i32,
    pub field50: u16,
    pub neg_response: Option<String>,
    pub unkstr3: Option<String>,
    pub unkstr4: Option<String>,
    pub output_presentations: Vec<Vec<DiagPreparation>>,
    pub diag_com_parameters: Vec<ComParameter>

}

impl DiagService {
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, base_addr: i64, pool_index: i32, parent_ecu: &ECU) -> Self {
        reader.seek(base_addr as usize);
        let mut bitflags = reader.read_u32().expect("Unable to read DiagService bitflags") as u64;
        let bitflags_ext = reader.read_u32().expect("Unable to read DiagService bitflags") as u64;

        let name = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);

        let name_ctf_idx = CReader::read_bitflag_i32(&mut bitflags, reader, -1);
        let desc_ctf_idx = CReader::read_bitflag_i32(&mut bitflags, reader, -1);

        let dataclass_servicetype = CReader::read_bitflag_u16(&mut bitflags, reader, 0);
        let dataclass_servicetype_shifted = 1 << (dataclass_servicetype - 1);

        let executable = CReader::read_bitflag_u16(&mut bitflags, reader, 0);
        let client_access_level = CReader::read_bitflag_u16(&mut bitflags, reader, 0);
        let security_access_level = CReader::read_bitflag_u16(&mut bitflags, reader, 0);

        let t_comparam_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let t_comparam_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let q_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let q_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let r_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let r_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let input_ref_name = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);

        let u_prep_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let u_prep_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let v_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let v_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let req_bytes_count = CReader::read_bitflag_i16(&mut bitflags, reader, 0);
        let req_bytes_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let w_outpres_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let w_outpres_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let field50 = CReader::read_bitflag_u16(&mut bitflags, reader, 0);

        let neg_response = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
        let unkstr3 = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
        let unkstr4 = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
        
        let p_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let p_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let diag_service_code_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let diag_service_code_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let s_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let s_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        bitflags = bitflags_ext;

        let x_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let x_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let y_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let y_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let z_count = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let z_offset = CReader::read_bitflag_i32(&mut bitflags, reader, 0);

        let req_bytes = match req_bytes_count {
            0 => Vec::new(),
            _ => {
                reader.seek(base_addr as usize + req_bytes_offset as usize);
                reader.read_bytes(req_bytes_count as usize).unwrap()
            }
        };

        let mut res = DiagService{
            name,
            name_ctf: lang.get_string(name_ctf_idx),
            desc_ctf: lang.get_string(desc_ctf_idx),
            prep: Vec::new(),
            is_executable: executable > 0,
            security_access_level,
            client_access_level,
            req_bytes_count: req_bytes_count as i32,
            req_bytes_offset,
            input_ref_name,
            dataclass_servicetype_shifted,
            p_count,
            pool_index,
            t_comparam_count,
            t_comparam_offset,
            q_count,
            q_offset,
            r_count,
            r_offset,
            req_bytes,
            v_count,
            v_offset,
            w_outpres_count,
            w_outpres_offset,
            field50,
            neg_response,
            unkstr3,
            unkstr4,
            output_presentations: Vec::new(),
            diag_com_parameters: Vec::new()
        };


        // Preperations steps
        res.prep = (0..u_prep_count as usize).map(|prep_index| {
            let pres_table_offset = base_addr + u_prep_offset as i64;
            reader.seek(pres_table_offset as usize + (prep_index*10));

            let prep_entry_offset = reader.read_i32().unwrap() as i64;
            let prep_entry_bitpos = reader.read_i32().unwrap();
            let prep_entry_mode = reader.read_i32().unwrap() as u16;
            DiagPreparation::new(reader, lang, pres_table_offset + prep_entry_offset, prep_entry_bitpos, prep_entry_mode, parent_ecu, &res)

        }).collect();

        // Output presentation formats
        let output_presentation_base_address = base_addr + res.w_outpres_offset as i64;
        (0..res.w_outpres_count).for_each(|pres_idx| {
            reader.seek((output_presentation_base_address + (pres_idx * 8) as i64) as usize);

            let res_pres_count = reader.read_i32().unwrap();
            let res_pres_offset = reader.read_i32().unwrap();

            let result_presentations: Vec<DiagPreparation> = (0..res_pres_count).map(|pres_inner_idx|{
                let pres_table_offset = output_presentation_base_address + res_pres_offset as i64;
                reader.seek((pres_table_offset + (pres_idx * 10) as i64) as usize);
                let prep_entry_offset = reader.read_i32().unwrap();
                let prep_entry_bit_offset = reader.read_i32().unwrap();
                let prep_entry_mode = reader.read_u16().unwrap();
                DiagPreparation::new(reader, lang, pres_table_offset as i64 + prep_entry_offset as i64, prep_entry_bit_offset, prep_entry_mode, parent_ecu, &res)
            }).collect();
            res.output_presentations.push(result_presentations);
        });


        // COM Parameters
        let com_param_table_offset = base_addr + t_comparam_offset as i64;
        res.diag_com_parameters = (0..t_comparam_count).map(|cp_index| {
            reader.seek((com_param_table_offset + (cp_index * 4) as i64) as usize);
            let res_cp_offset = reader.read_i32().unwrap();
            let cp_entry_base_addr = com_param_table_offset + res_cp_offset as i64;
            ComParameter::new(reader, cp_entry_base_addr, &parent_ecu.ecu_ifaces[1])
        }).collect();

        // Diagnostic codes
        let dtc_pool = parent_ecu.parent_container.cff_header.dsc_pool.clone();
        let dtc_table_base_address = base_addr + diag_service_code_offset as i64;

        let mut dtc_reader = raf::Raf::from_bytes(&dtc_pool, raf::RafByteOrder::LE);
        (0..diag_service_code_count).for_each(|dtc_index| {
            reader.seek((dtc_table_base_address + (4*dtc_index) as i64) as usize);
            let dtc_entry_base_address = reader.read_i32().unwrap() as i64 + dtc_table_base_address;
            reader.seek(dtc_entry_base_address as usize);

            let mut dtc_entry_bit_flags = reader.read_u16().unwrap() as u64;
            let idk1 = CReader::read_bitflag_u8(&mut dtc_entry_bit_flags, reader, 0);
            let idk2 = CReader::read_bitflag_u8(&mut dtc_entry_bit_flags, reader, 0);
            let dtc_pool_offset = CReader::read_bitflag_i32(&mut dtc_entry_bit_flags, reader, 0);
            let dtc_qualifier = CReader::read_bitflag_string(&mut dtc_entry_bit_flags, reader, dtc_entry_base_address);

            dtc_reader.seek(dtc_pool_offset as usize * 8);

            let dtc_record_offset = dtc_reader.read_i32().unwrap() as i64 + parent_ecu.parent_container.cff_header.dsc_block_offset;
            let dtc_record_size = dtc_reader.read_i32().unwrap();

            reader.seek(dtc_record_offset as usize);

            println!("{:?}", dtc_qualifier);//, reader.read_bytes(dtc_record_size as usize).unwrap());
        });

        res
    }   
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Dump {
    Text(Option<String>),
    Data(Vec<u8>)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferredDataType {
    UnassignedType,
    IntegerType,
    NativeInfoPoolType,
    NativePresentationType,
    UnhandledITType,
    UnhandledSP17Type,
    UnhandledType,
    BitDumpType,
    ExtendedBitDumpType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagPreparation{
    //parent_ecu: &'a ECU<'a>,
    //parent_diag_service: &'a DiagService,
    dump: Dump,
    name: Option<String>,
    mode_cfg: u16,
    alternative_bit_width: i32,
    result_bit_size: i32,
    data_type: InferredDataType,
    system_param: i32,
    info_pool_idx: i32,
    size_in_bits: i32,
    pres_pool_idx: i32,
    bit_pos: i32,
    name_ctf: Option<String>,
    unk1: u8,
    unk2: u8,
    iit_offset: i32,
    field_1e: i32
}


impl DiagPreparation {
    // parent_diag_service: &'a mut DiagService
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, base_addr: i64, bit_pos: i32, mode_cfg: u16, parent_ecu: &ECU, parent_diag_service: &DiagService) -> Self {
            
        reader.seek(base_addr as usize);
            let mut bitflags = reader.read_u32().expect("unable to read bitflags!") as u64;

            let name = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
            let name_ctf_idx = CReader::read_bitflag_i32(&mut  bitflags, reader, -1);
            let unk1 = CReader::read_bitflag_u8(&mut  bitflags, reader, 0);
            let unk2 = CReader::read_bitflag_u8(&mut  bitflags, reader, 0);
            let alternative_bit_width = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let iit_offset = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let info_pool_idx = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let pres_pool_idx = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let field_1e = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let system_param = CReader::read_bitflag_i16(&mut  bitflags, reader, -1) as i32;
            let dump_mode = CReader::read_bitflag_i16(&mut  bitflags, reader, 0);
            let dump_size = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            
            let dump = match dump_mode {
                5 => {
                    match CReader::read_bitflag_dump(&mut bitflags, reader, dump_size, base_addr) {
                        Some(bytes) => Dump::Text(Some(String::from_utf8(bytes).unwrap())),
                        None => Dump::Text(None)
                    }
                },
                _ => Dump::Data(CReader::read_bitflag_dump(&mut bitflags, reader, dump_size, base_addr).unwrap_or(Vec::new()))
            };
            let mut res = DiagPreparation {
                name,
                //parent_diag_service,
                //parent_ecu,
                dump,
                mode_cfg,
                alternative_bit_width,
                result_bit_size: 0,
                data_type: InferredDataType::UnassignedType,
                size_in_bits: 0,
                system_param,
                info_pool_idx,
                pres_pool_idx,
                bit_pos,
                name_ctf: lang.get_string(name_ctf_idx),
                unk1,
                unk2,
                field_1e,
                iit_offset
                
            };
            res.get_size_in_bits(reader, parent_ecu, parent_diag_service);
            res
    }

    fn get_size_in_bits(&mut self, reader: &mut raf::Raf, parent_ecu: &ECU, parent_diag_service: &DiagService) {
        let mode_e = (self.mode_cfg as u32) & 0xF000;
        let mode_h = (self.mode_cfg as u32) & 0xFF0;
        let mode_l = (self.mode_cfg as u32) & 0xF;
        let mut result_bit_size: i32 = 0;

        if self.mode_cfg & 0xF00 == 0x300 {
            if mode_l > 6 {
                panic!("nImplType <= 6")
            }

            match mode_h {

                0x320 => {
                    result_bit_size = INT_SIZE_MAPPING[mode_l as usize] as i32;
                    self.data_type = InferredDataType::IntegerType;
                },
                0x330 => {
                    result_bit_size = self.alternative_bit_width;
                    self.data_type = InferredDataType::BitDumpType;
                },
                0x340 => {
                    self.data_type = InferredDataType::UnhandledITType;
                },
                _ => {}
            }
        } else {
            if self.system_param == -1 { // Default type

                if mode_e == 0x8000 {
                    self.data_type = InferredDataType::NativeInfoPoolType;

                    let info_block = &parent_ecu.info;
                    let pool_bytes = ECU::read_ecu_pool(reader, &info_block);

                    let mut pool_reader = raf::Raf::from_bytes(&pool_bytes, raf::RafByteOrder::LE);

                    pool_reader.seek(info_block.entry_size as usize * self.info_pool_idx as usize);

                    let presentation_struct_offset = pool_reader.read_i32().unwrap();
                    let presentation_struct_size = pool_reader.read_i32().unwrap();

                    reader.seek((presentation_struct_offset + (info_block.block_offset)) as usize);

                    let presentation_struct = reader.read_bytes(presentation_struct_size as usize).expect("Error reading presentation structure!");
                
                    let presentation_mode = read_cbf_with_offset(0x1C, &StructureName::PRESENTATION_STRUCTURE, &presentation_struct); // Type
                    let presentation_length = read_cbf_with_offset(0x1A, &StructureName::PRESENTATION_STRUCTURE, &presentation_struct); // Length

                    if presentation_length > 0 {
                        result_bit_size = presentation_length;
                    } else {
                        result_bit_size = read_cbf_with_offset(0x21, &StructureName::PRESENTATION_STRUCTURE, &presentation_struct);
                    }
                    if presentation_mode == 0 {
                        result_bit_size *= 8; // Bytes to bits
                    }
                } else if mode_e == 0x2000 {
                    self.data_type = InferredDataType::NativePresentationType;

                    let pres_block = &parent_ecu.presentations;
                    let pool_bytes = ECU::read_ecu_pool(reader, &pres_block);

                    let mut pool_reader = raf::Raf::from_bytes(&pool_bytes, raf::RafByteOrder::LE);

                    pool_reader.seek(pres_block.entry_size as usize * self.pres_pool_idx as usize);

                    let presentation_struct_offset = pool_reader.read_i32().unwrap();
                    let presentation_struct_size = pool_reader.read_i32().unwrap();

                    reader.seek((presentation_struct_offset + (pres_block.block_offset)) as usize);
                    let presentation_struct = reader.read_bytes(presentation_struct_size as usize).expect("Error reading presentation structure!");
                
                    let presentation_mode = read_cbf_with_offset(0x1C, &StructureName::PRESENTATION_STRUCTURE, &presentation_struct); // Type
                    let presentation_length = read_cbf_with_offset(0x1A, &StructureName::PRESENTATION_STRUCTURE, &presentation_struct); // Length

                    if presentation_length > 0 {
                        result_bit_size = presentation_length;
                    } else {
                        result_bit_size = read_cbf_with_offset(0x21, &StructureName::PRESENTATION_STRUCTURE, &presentation_struct);
                    }
                    if presentation_mode == 0 {
                        result_bit_size *= 8; // Bytes to bits
                    }
                } else {
                    panic!("Unknown system type for {}", self.name.clone().unwrap());
                }
            } else {
                if mode_h == 0x410 {
                    let reduced_sys_param = self.system_param - 0x10;
                    if reduced_sys_param == 0 {
                        let res_byte_size = ((parent_diag_service.req_bytes_count & 0xFF) - (self.bit_pos / 8)) * 8;
                        self.data_type = InferredDataType::ExtendedBitDumpType;
                        // TODO 
                    } else if reduced_sys_param == 17 {
                        let parent = parent_ecu.diag_services.clone();
                        if let Some(find) = parent.into_iter().find(|mut x| { x.name.as_ref().unwrap() == parent_diag_service.input_ref_name.as_ref().unwrap() }) {
                            let has_request_data = find.req_bytes_count > 0;
                            let mut internal_data_type = find.dataclass_servicetype_shifted;

                            if (find.dataclass_servicetype_shifted & 0xC) > 0 && has_request_data {
                                internal_data_type = match find.dataclass_servicetype_shifted & 4 {
                                    x if x > 0 => 0x10000000,
                                    _ => 0x20000000
                                }
                            }

                            self.data_type = InferredDataType::UnhandledSP17Type;
                            if internal_data_type & 0x10000 != 0 {
                                result_bit_size = find.p_count;
                            } else {
                                result_bit_size = parent_diag_service.req_bytes_count * 8;
                            }
                        }
                    }
                } else if mode_h == 0x420 {
                    if mode_l > 6 {
                        panic!("Tryping to map a data type that cannot exist");
                    }
                    self.data_type = InferredDataType::IntegerType;
                    result_bit_size = INT_SIZE_MAPPING[mode_l as usize] as i32;
                } else if mode_h == 0x430 {
                    result_bit_size = self.alternative_bit_width;
                    self.data_type = InferredDataType::BitDumpType
                }
            }
        }
        self.result_bit_size = result_bit_size
    }
}