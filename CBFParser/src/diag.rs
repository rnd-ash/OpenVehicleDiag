 use crate::ecu::*;
 use crate::caesar::*;
 use crate::cxf::*;
 use common::raf;
 use crate::structure::*;

const INT_SIZE_MAPPING: [u8; 7] = [0x00, 0x01, 0x04, 0x08, 0x10, 0x20, 0x40];

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

#[derive(Debug)]
pub struct DiagService{
    name: Option<String>,
    prep: Vec<DiagPreparation>,
    is_executable: bool,
    client_access_level: u16,
    security_access_level: u16
}

impl DiagService {
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, base_addr: i64, pool_index: i32, parent_ecu: &ECU) -> Self {
        reader.seek(base_addr as usize);
        let mut bitflags = reader.read_u32().expect("Unable to read DiagService bitflags") as u64;
        let bitflags_ext = reader.read_u32().expect("Unable to read DiagService bitflags") as u64;

        let name = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);

        let name_ctf = CReader::read_bitflag_i32(&mut bitflags, reader, -1);
        let desc_ctf = CReader::read_bitflag_i32(&mut bitflags, reader, -1);

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

        let inputrefnamemaybe = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);

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

        let prep: Vec<DiagPreparation> = (0..u_prep_count as usize).map(|prep_index| {
            let pres_table_offset = base_addr + u_prep_offset as i64;
            reader.seek(pres_table_offset as usize + (prep_index*10));

            let prep_entry_offset = reader.read_i32().unwrap() as i64;
            let prep_entry_bitpos = reader.read_i32().unwrap();
            let prep_entry_mode = reader.read_i32().unwrap() as u16;

            DiagPreparation::new(reader, lang, pres_table_offset + prep_entry_offset, prep_entry_bitpos, prep_entry_mode, parent_ecu)

        }).collect();

        DiagService{
            name,
            prep,
            is_executable: executable > 0,
            security_access_level,
            client_access_level
        }
    }
}

#[derive(Debug)]
pub enum Dump {
    Text(Option<String>),
    Data(Vec<u8>)
}


#[derive(Debug)]
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

#[derive(Debug)]
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
    info_pool_idx: i32
}


impl DiagPreparation {
    // parent_diag_service: &'a mut DiagService
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, base_addr: i64, bit_pos: i32, mode_cfg: u16, parent_ecu: &ECU) -> Self {
            
        reader.seek(base_addr as usize);
            let mut bitflags = reader.read_u32().expect("unable to read bitflags!") as u64;

            let name = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
            let name_ctf = CReader::read_bitflag_i32(&mut  bitflags, reader, -1);
            let unk1 = CReader::read_bitflag_u8(&mut  bitflags, reader, 0);
            let unk2 = CReader::read_bitflag_u8(&mut  bitflags, reader, 0);
            let alternative_bit_width = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let iit_offset = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let info_pool_idx = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
            let pre_pool_idx = CReader::read_bitflag_i32(&mut  bitflags, reader, 0);
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
                system_param,
                info_pool_idx
                
            };
            res.get_size_in_bits(reader, parent_ecu);
            res
    }

    fn get_size_in_bits(&mut self, reader: &mut raf::Raf, parent_ecu: &ECU) {
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
                }

            }
        }
    }
}