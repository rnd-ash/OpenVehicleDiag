use common::raf;
use crate::caesar::{CReader, CContainer};
use crate::cxf::*;
use crate::diag::*;
use serde::*;
use serde_json::*;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct block {
    pub block_offset: i32,
    pub entry_count: i32,
    pub entry_size: i32,
    pub block_size: i32
}

impl block {
    pub fn new(reader: &mut raf::Raf, bit_flag: &mut u64, offset: i32) -> Self {
        Self {
            block_offset: CReader::read_bitflag_i32(bit_flag, reader, 0) + offset,
            entry_count: CReader::read_bitflag_i32(bit_flag, reader, 0),
            entry_size: CReader::read_bitflag_i32(bit_flag, reader, 0),
            block_size: CReader::read_bitflag_i32(bit_flag, reader, 0),
        }
    }
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamName {
    /// Bus baud speed
    CP_BAUDRATE,
    /// CAN ID for sending Diag requests to global bus (Interior CAN only)
    CP_GLOBAL_REQUEST_CANIDENTIFIER,
    /// ??
    CP_FUNCTIONAL_REQUEST_CANIDENTIFIER,
    /// ECU Request CAN ID (Requests to ECU have this ID)
    CP_REQUEST_CANIDENTIFIER,
    /// ECU Response CAN ID (It writes data with this ID)
    CP_RESPONSE_CANIDENTIFIER,
    CP_PARTNUMBERID,
    CP_PARTBLOCK,
    CP_HWVERSIONID,
    CP_SWVERSIONID,
    CP_SWVERSIONBLOCK,
    CP_SUPPLIERID,
    CP_SWSUPPLIERBLOCK,
    CP_ADDRESSMODE,
    CP_ADDRESSEXTENSION,
    CP_ROE_RESPONSE_CANIDENTIFIER,
    /// Shouldn't all ISO15765 messages use the ECUs timing?
    CP_USE_TIMING_RECEIVED_FROM_ECU,
    /// ISO15765 minimum seperation time in MS 
    CP_STMIN_SUG,
    CP_BLOCKSIZE_SUG,
    CP_P2_TIMEOUT,
    CP_S3_TP_PHYS_TIMER,
    CP_S3_TP_FUNC_TIMER,
    CP_BR_SUG,
    CP_CAN_TRANSMIT,
    /// Max block size for ISO15765
    CP_BS_MAX,
    CP_CS_MAX,
    CPI_ROUTINECOUNTER,
    CP_REQREPCOUNT,
    // looks like outliers?
    CP_P2_EXT_TIMEOUT_7F_78,
    CP_P2_EXT_TIMEOUT_7F_21,

    UNKNOWN
}

impl std::fmt::Display for ParamName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ParamName {
    pub fn from_string(txt: &str) -> Self {
        match txt {
            "CP_BAUDRATE" => ParamName::CP_BAUDRATE,
            "CP_GLOBAL_REQUEST_CANIDENTIFIER" => ParamName::CP_GLOBAL_REQUEST_CANIDENTIFIER,
            "CP_FUNCTIONAL_REQUEST_CANIDENTIFIER" => ParamName::CP_FUNCTIONAL_REQUEST_CANIDENTIFIER,
            "CP_REQUEST_CANIDENTIFIER" => ParamName::CP_REQUEST_CANIDENTIFIER,
            "CP_RESPONSE_CANIDENTIFIER" => ParamName::CP_RESPONSE_CANIDENTIFIER,
            "CP_PARTNUMBERID" => ParamName::CP_PARTNUMBERID,
            "CP_PARTBLOCK" => ParamName::CP_PARTBLOCK,
            "CP_HWVERSIONID" => ParamName::CP_HWVERSIONID,
            "CP_SWVERSIONID" => ParamName::CP_SWVERSIONID,
            "CP_SWVERSIONBLOCK" => ParamName::CP_SWVERSIONBLOCK,
            "CP_SUPPLIERID" => ParamName::CP_SUPPLIERID,
            "CP_SWSUPPLIERBLOCK" => ParamName::CP_SWSUPPLIERBLOCK,
            "CP_ADDRESSMODE" => ParamName::CP_ADDRESSMODE,
            "CP_ADDRESSEXTENSION" => ParamName::CP_ADDRESSEXTENSION,
            "CP_ROE_RESPONSE_CANIDENTIFIER" => ParamName::CP_ROE_RESPONSE_CANIDENTIFIER,
            "CP_USE_TIMING_RECEIVED_FROM_ECU" => ParamName::CP_USE_TIMING_RECEIVED_FROM_ECU,
            "CP_STMIN_SUG" => ParamName::CP_STMIN_SUG,
            "CP_BLOCKSIZE_SUG" => ParamName::CP_BLOCKSIZE_SUG,
            "CP_P2_TIMEOUT" => ParamName::CP_P2_TIMEOUT,
            "CP_S3_TP_PHYS_TIMER" => ParamName::CP_S3_TP_PHYS_TIMER,
            "CP_S3_TP_FUNC_TIMER" => ParamName::CP_S3_TP_FUNC_TIMER,
            "CP_BR_SUG" => ParamName::CP_BR_SUG,
            "CP_CAN_TRANSMIT" => ParamName::CP_CAN_TRANSMIT,
            "CP_BS_MAX" => ParamName::CP_BS_MAX,
            "CP_CS_MAX" => ParamName::CP_CS_MAX,
            "CPI_ROUTINECOUNTER" => ParamName::CPI_ROUTINECOUNTER,
            "CP_REQREPCOUNT" => ParamName::CP_REQREPCOUNT,
            "CP_P2_EXT_TIMEOUT_7F_78" => ParamName::CP_P2_EXT_TIMEOUT_7F_78,
            "CP_P2_EXT_TIMEOUT_7F_21" => ParamName::CP_P2_EXT_TIMEOUT_7F_21,
            _ => {
                println!("WARNING: UNKNOWN COM PARAM: '{}'", txt);
                ParamName::UNKNOWN
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComParameter {
    pub param_index: i32,
    pub unk3: i32,
    pub sub_iface_index: i32,
    pub unk5: i32,
    pub unk_ctf: i32,
    pub phrase: i32,
    pub dump_size: i32,
    pub dump: Vec<u8>,

    pub com_param_value: i32,
    pub com_param_name: Option<String>,
    pub base_addr: i64,
}

impl ComParameter {
    pub fn new(reader: &mut raf::Raf, base_addr: i64, parent_iface: &ECUInterface) -> Self {
        
        reader.seek(base_addr as usize);
        let mut bitflags = reader.read_u16().expect("Error reading bitflags") as u64;

        let param_index = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        let unk3 = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        let sub_iface_index = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        let unk5 = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        let unk_ctf = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let phrase = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        let dump_size = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        let dump = CReader::read_bitflag_dump(&mut bitflags, reader, dump_size, base_addr).unwrap();
        let mut com_param_value = 0;

        if dump_size == 4 {
            com_param_value = (dump[3] as i32) << 24 | (dump[2] as i32) << 16 | (dump[1] as i32) << 8 | (dump[0] as i32)
        }
        let com_param_name = parent_iface.com_parameters.get(param_index as usize).map(|x| x.clone());

        Self {
            param_index,
            unk3,
            sub_iface_index,
            unk5,
            unk_ctf,
            phrase,
            dump_size,
            dump,
            com_param_value,
            com_param_name,
            base_addr,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUInterfaceSubType {
    pub name: String,
    pub name_ctf: Option<String>,
    pub desc_ctf: Option<String>,
    pub base_addr: i64,
    pub index: i32,
    pub unk3: i32,
    pub unk4: i32,
    pub unk5: i32,
    pub unk6: i32,
    pub unk7: i32,
    pub unk8: i32,
    pub unk9: i32,
    pub unk10: i32,
    pub com_params: Vec<ComParameter>
}

impl ECUInterfaceSubType {
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, base_addr: i64, index: i32) -> Self {
        reader.seek(base_addr as usize);

        let mut bitflags = reader.read_u32().expect("Error reading iface bitflag") as u64;

        Self {
            index: index,
            base_addr: base_addr,
            name: CReader::read_bitflag_string(&mut bitflags, reader, base_addr).unwrap(),
            name_ctf: lang.get_string(CReader::read_bitflag_i32(&mut bitflags, reader, -1)),
            desc_ctf: lang.get_string(CReader::read_bitflag_i32(&mut bitflags, reader, -1)),

            unk3: CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32,
            unk4: CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32,

            unk5: CReader::read_bitflag_i32(&mut bitflags, reader, 0),
            unk6: CReader::read_bitflag_i32(&mut bitflags, reader, 0),
            unk7: CReader::read_bitflag_i32(&mut bitflags, reader, 0),

            unk8: CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32,
            unk9: CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32,
            unk10: CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32,
            com_params: Vec::new()
        }
    }

    pub fn get_com_param(&self, name: &str) -> Option<&ComParameter> {
        self.com_params.iter().find(|x| x.com_param_name.clone().unwrap_or("N/A".to_string()) == name)
    }

    pub fn get_com_param_value(&self, name: ParamName) -> Option<i32> {
        self.get_com_param(name.to_string().as_str()).map(|x| x.com_param_value)
    }

    pub fn get_com_param_value_res(&self, name: ParamName, result: &mut i32) -> bool {
        let param = self.get_com_param(name.to_string().as_str());

        match param {
            Some(p) => {
                *result = p.com_param_value;
                true
            },
            None => {
                *result = 0;
                false
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVarientPattern {
        unk_buffer_size: i32,
        unk_buffer: Vec<u8>,
        unk_3: i32,
        unk_4: i32,
        unk_5: i32,
        vendor_name: Option<String>,
        unk_7: i32,
        unk_8: i32,
        unk_9: i32,
        unk_10: i32,

        unk_11: i32,
        unk_12: i32,
        unk_13: i32,
        unk_14: i32,
        unk_15: i32,
        unk_16: Vec<u8>,
        unk_17: i32,
        unk_18: i32,
        unk_19: i32,
        unk_20: i32,
        unk_21: Option<String>,
        unk_22: i32,
        unk_23: i32,
        variant_id: i32,
        pattern_type: i32,
        base_addr: i64,
}

impl ECUVarientPattern {
    pub fn new(reader: &mut raf::Raf, base_addr: i64) -> Self {
        reader.seek(base_addr as usize);
        let mut bitflags = reader.read_u32().unwrap() as u64;
        let mut ret: ECUVarientPattern = unsafe { std::mem::zeroed() };

        ret.unk_buffer_size = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.unk_buffer = CReader::read_bitflag_dump(&mut bitflags, reader, ret.unk_buffer_size, base_addr).unwrap_or(Vec::new());
        ret.unk_3 = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.unk_4 = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.unk_5 = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.vendor_name = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
        ret.unk_7 = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        ret.unk_8 = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        ret.unk_9 = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        ret.unk_10 = CReader::read_bitflag_i16(&mut bitflags, reader, 0) as i32;
        ret.unk_11 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_12 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_13 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_14 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_15 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_16 = CReader::read_bitflag_dump(&mut bitflags, reader, 5, base_addr).unwrap_or(Vec::new()); // read with a constant size
        ret.unk_17 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_18 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_19 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_20 = CReader::read_bitflag_u8(&mut bitflags, reader, 0) as i32;
        ret.unk_21 = CReader::read_bitflag_string(&mut bitflags, reader, base_addr);
        ret.unk_22 = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.unk_23 = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.variant_id = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret.pattern_type = CReader::read_bitflag_i32(&mut bitflags, reader, 0);
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUVarient {
    name: Option<String>,
    name_ctf: i32,
    desc_ctf: i32,
    unk_str1: Option<String>,
    unk_str2: Option<String>,
    unk1: i32,

    matching_pattern_count: i32,// A
    matching_pattern_offset: i32,
    subsection_b_count: i32,// B
    subsection_b_offset: i32,
    com_param_count: i32,// C
    com_param_offset: i32,
    subsection_d_count: i32,// D
    subsection_d_offset: i32,
    diag_services_count: i32,// E
    diag_services_offset: i32,
    subsection_f_count: i32,// F
    subsection_f_offset: i32,
    subsection_g_count: i32,// G
    subsection_g_offset: i32,
    subsection_h_count: i32,// H
    subsection_h_offset: i32,
    VCDomainsCount: i32,// I
    VCDomainsOffset: i32,

    negative_resp_name: String,
    unk_byte: i32,

    vc_domain_pool_offsets: Vec<i32>,
    diag_services_pool_offsets: Vec<i32>,

    vc_domains: Vec<VCDomain>,
    varient_patterns: Vec<ECUVarientPattern>,
    diag_services: Vec<DiagService>,
    base_addr: i64
}

impl ECUVarient {
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, parent_ecu: &mut ECU, base_addr: i64, block_size: i32) -> Self {
        reader.seek(base_addr as usize);

        let varient_bytes = reader.read_bytes(block_size as usize).expect("Error reading ECU Varient bytes");

        let mut varreader = raf::Raf::from_bytes(&varient_bytes, raf::RafByteOrder::LE);

        let mut ret: ECUVarient = unsafe { std::mem::zeroed() };
        let mut bitflags = varreader.read_u32().unwrap() as u64;
        let skip = varreader.read_i32().unwrap();
        ret.base_addr = base_addr;
        ret.name = CReader::read_bitflag_string(&mut bitflags, &mut varreader, 0);
        ret.name_ctf = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, -1);
        ret.desc_ctf = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, -1);
        ret.unk_str1 = CReader::read_bitflag_string(&mut bitflags, &mut varreader, 0);
        ret.unk_str2 = CReader::read_bitflag_string(&mut bitflags, &mut varreader, 0);

        ret.unk1 = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);

        ret.matching_pattern_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.matching_pattern_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_b_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_b_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.com_param_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.com_param_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_d_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_d_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.diag_services_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.diag_services_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_f_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_f_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_g_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_g_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_h_count = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.subsection_h_offset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);

        ret.VCDomainsCount = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);
        ret.VCDomainsOffset = CReader::read_bitflag_i32(&mut bitflags, &mut varreader, 0);

        ret.negative_resp_name = CReader::read_bitflag_string(&mut bitflags, &mut varreader, 0).unwrap_or(String::new());

        ret.unk_byte = CReader::read_bitflag_i8(&mut bitflags, &mut varreader, 0) as i32;
        varreader.seek(ret.VCDomainsOffset as usize);

        ret.vc_domain_pool_offsets = (0..ret.VCDomainsCount).map(|i| {varreader.read_i32().unwrap()}).collect();

        varreader.seek(ret.diag_services_offset as usize);
        ret.diag_services_pool_offsets = (0..ret.VCDomainsCount).map(|i| {varreader.read_i32().unwrap()}).collect();


        ret.create_vc_domains(reader, parent_ecu, lang);
        ret.create_diag_services(reader, parent_ecu, lang);
        ret.create_var_patterns(reader);
        ret.create_com_params(reader, parent_ecu);

        ret
    }

    fn create_vc_domains(&mut self, reader: &mut raf::Raf, parent_ecu: &ECU, lang: &CTFLanguage) {

    }

    fn create_diag_services(&mut self, reader: &mut raf::Raf, parent_ecu: &ECU, lang: &CTFLanguage) {
        let mut diag_copy = parent_ecu.diag_services.clone().into_iter();
        self.diag_services = self.diag_services_pool_offsets.clone().into_iter().map(|idx| {
            diag_copy.find(|x| {x.pool_index == idx })
        }).filter(|l| l.is_some())
        .map(|x| x.unwrap()).collect();
    }

    fn create_var_patterns(&mut self, reader: &mut raf::Raf) {
        let table_offset = self.base_addr + self.matching_pattern_offset as i64;
        reader.seek(table_offset as usize);

        self.varient_patterns = (0..self.matching_pattern_count).map(|pattern_index| {
            reader.seek((table_offset + (pattern_index*4) as i64) as usize);

            let pattern_offset = reader.read_i32().unwrap();
            let pattern_address = pattern_offset as i64 + table_offset;
            ECUVarientPattern::new(reader, pattern_address)

        }).collect();
    }

    fn create_com_params(&mut self, reader: &mut raf::Raf, parent_ecu: &mut ECU) {
        let com_param_base_address = self.base_addr + self.com_param_offset as i64;
        reader.seek(com_param_base_address as usize);
        let com_param_offsets: Vec<i64> = (0..self.com_param_count).map(|_| reader.read_i32().unwrap() as i64 + com_param_base_address).collect();

        parent_ecu.ecu_ifaces.clone().iter().for_each(|iface| {
            let mut clone = parent_ecu.clone();
            com_param_offsets.clone().into_iter().for_each(|com_offset| {
                let cp = ComParameter::new(reader, com_offset, iface);
                eprintln!("{:?}", cp);
                clone.ecu_ifaces_subtype[cp.sub_iface_index as usize].com_params.push(cp);
            });
            *parent_ecu = clone;
        });
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VCDomain{}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECUInterface {
    pub name: Option<String>,
    pub name_ctf: i32,
    pub desc_ctf: i32,
    pub version_str: Option<String>,
    pub com_param_count: i32,
    pub com_param_list_offset: i32,
    pub unk6: i32,
    pub com_parameters: Vec<String>,
    pub base_addr: i64,
    pub version: i32
}

impl ECUInterface {
    pub fn new(reader: &mut raf::Raf, base_addr: i64) -> Self {
        reader.seek(base_addr as usize);

        let mut iface_bf = reader.read_i32().expect("Error reading ECU Bitflag") as u64;

        let name= CReader::read_bitflag_string(&mut iface_bf, reader, base_addr);
        let name_ctf= CReader::read_bitflag_i32(&mut iface_bf, reader, -1);
        let desc_ctf= CReader::read_bitflag_i32(&mut iface_bf, reader, -1);
        let version_str= CReader::read_bitflag_string(&mut iface_bf, reader, base_addr);
        let version= CReader::read_bitflag_i32(&mut iface_bf, reader, 0);
        let com_param_count= CReader::read_bitflag_i32(&mut iface_bf, reader, 0);
        let com_param_list_offset= CReader::read_bitflag_i32(&mut iface_bf, reader, 0);
        let unk6= CReader::read_bitflag_i32(&mut iface_bf, reader, 0);

        let com_param_foffset = com_param_list_offset as i64 + base_addr;

        let com_parameters: Vec<String> = (0..com_param_count).map(|str_index|{
            reader.seek((com_param_foffset + (str_index*4) as i64) as usize);

            let iface_read_ptr = reader.read_i32().unwrap() as i64 + com_param_foffset;
            reader.seek(iface_read_ptr as usize);
            CReader::read_string(reader)
        }).collect();

        Self {
            name,
            name_ctf,
            desc_ctf,
            version_str,
            version,
            com_param_count,
            com_param_list_offset,
            unk6,
            com_parameters,
            base_addr
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECU {
    pub name: String,
    pub ecuname_ctf: Option<String>,
    pub ecudesc_ctf: Option<String>,
    pub xml_version: String,
    pub interface_block_count: i32,
    pub interface_table_offset: i32,
    pub sub_interface_count: i32,
    pub sub_interface_offset: i32,
    pub ecu_class_name: String,
    pub unk_str7: String,
    pub unk_str8: String,

    // ----
    pub ignition_required: i32,
    pub unk2: i32,
    pub unk_block_count: i32,
    pub unk_block_offset: i32,
    pub ecu_sgml_src: i32,
    pub unk6_relative_offset: i32,

    // ECU Blocks
    pub ecuvarient: block,
    pub diagjob: block,
    pub dtc: block,
    pub env: block,
    pub vcdomain: block,
    pub presentations: block,
    pub info: block,
    pub unk_block: block,

    // --
    pub unk39: i32,

    pub parent_container: CContainer,

    pub ecu_ifaces: Vec<ECUInterface>,
    pub ecu_ifaces_subtype: Vec<ECUInterfaceSubType>,
    pub ecu_varient: Vec<ECUVarient>,
    pub diag_services: Vec<DiagService>,

    //pub cache_varcoding: Vec<u8>,
    //pub cache_varient: Vec<u8>,
    //pub cache_diagjob: Vec<u8>,
    //pub cache_infopool: Vec<u8>,
    //pub cache_prespool: Vec<u8>,
    //pub cache_envpool: Vec<u8>,

    pub base_addr: i64,
}

impl ECU {
    pub fn new(reader: &mut raf::Raf, lang: &CTFLanguage, header: &CFFHeader, base_addr: i64, pcontainer: CContainer) -> ! {

        let mut ecu_bitflags = reader.read_u32().expect("Error reading ECU Bitflag") as u64;
        let ecu_bitflags_ext = reader.read_i16().expect("Error reading ECU Ext Bitflag") as u64;

        ecu_bitflags = ecu_bitflags | ecu_bitflags_ext << 32;

        println!("Skip {:?}", reader.read_i32());


        let name = CReader::read_bitflag_string(&mut ecu_bitflags, reader, base_addr);
        let ecuname_ctf_idx = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, -1);
        let ecudesc_ctf_idx = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, -1);
        let xml_version = CReader::read_bitflag_string(&mut ecu_bitflags, reader, base_addr);
        let iface_blockcount = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);
        let iface_tableoffset = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);
        let subiface_count = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);
        let subiface_offset = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);
        let ecu_classname = CReader::read_bitflag_string(&mut ecu_bitflags, reader, base_addr);
        let unk7 = CReader::read_bitflag_string(&mut ecu_bitflags, reader, base_addr);
        let unk8 = CReader::read_bitflag_string(&mut ecu_bitflags, reader, base_addr);

        let data_offset = header.size_of_str_pool + STUB_HEADER_SIZE as i32 + header.cff_header_size + 4;

        let ign_required = CReader::read_bitflag_i16(&mut ecu_bitflags, reader, 0);
        let unk2 = CReader::read_bitflag_i16(&mut ecu_bitflags, reader, 0);

        let unk_blockcount = CReader::read_bitflag_i16(&mut ecu_bitflags, reader, 0);
        let unk_blockoffset = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);
        let ecu_sgml_src = CReader::read_bitflag_i16(&mut ecu_bitflags, reader, 0);
        let unk6_rel_offset = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);

        let ecuvarient_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let diagjob_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let dtc_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let vc_domain_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let env_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let pres_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let info_block = block::new(reader, &mut ecu_bitflags, data_offset);
        let unk_block = block::new(reader, &mut ecu_bitflags, data_offset);
        
        let unk39 = CReader::read_bitflag_i32(&mut ecu_bitflags, reader, 0);

        let iface_table_addr = base_addr + iface_tableoffset as i64;

        let ecu_interfaces: Vec<ECUInterface> = (0..iface_blockcount).map(|iface_buff_index| {
            reader.seek((iface_table_addr + (iface_buff_index*4) as i64) as usize);
            let iface_blockoffset = reader.read_i32().unwrap();
            let ecu_iface_baseaddr = iface_table_addr + iface_blockoffset as i64;
            ECUInterface::new(reader, ecu_iface_baseaddr)

        }).collect();

        let ct_table_addr = (base_addr + subiface_offset as i64) as usize;
        let ecu_subinterfaces: Vec<ECUInterfaceSubType> = (0..subiface_count as usize).map(|buf_index| {
            reader.seek(ct_table_addr + (buf_index*4));
            let actual_blk_offset = reader.read_i32().unwrap();
            let ct_base_addr = ct_table_addr as i64 + actual_blk_offset as i64;

            ECUInterfaceSubType::new(reader, lang, ct_base_addr, buf_index as i32)

        }).collect();

        let mut res = Self {
            base_addr,
            ecu_ifaces: ecu_interfaces,
            ecu_ifaces_subtype: ecu_subinterfaces,
            ecuvarient: ecuvarient_block,
            diagjob: diagjob_block,
            dtc: dtc_block,
            env: env_block,
            vcdomain: vc_domain_block,
            presentations: pres_block,
            info: info_block,
            unk_block: unk_block,
            
            ignition_required: ign_required as i32,
            unk2: unk2 as i32,
            unk_block_count: unk_blockcount as i32,
            unk_block_offset: unk_blockoffset,
            ecu_sgml_src: ecu_sgml_src as i32,
            unk6_relative_offset: unk6_rel_offset,
            ecu_varient: Vec::new(),
            diag_services: Vec::new(),

            name: name.unwrap(),
            ecuname_ctf: lang.get_string(ecuname_ctf_idx),
            ecudesc_ctf: lang.get_string(ecudesc_ctf_idx),
            xml_version: xml_version.unwrap(),
            interface_block_count: iface_blockcount,
            interface_table_offset: iface_tableoffset,
            sub_interface_count: subiface_count,
            sub_interface_offset: subiface_offset,
            ecu_class_name: ecu_classname.unwrap(),
            unk_str7: unk7.unwrap_or(format!("N/A")),
            unk_str8: unk8.unwrap_or(format!("Unknown")),

            unk39: unk39,

            parent_container: pcontainer
        };

        res.create_diag_pool(reader, lang);
        res.create_ecu_varients(reader, lang);

        println!("{}", serde_json::to_string_pretty(&res).unwrap());
        panic!("Done")
    }

    pub fn create_diag_pool(&mut self, reader: &mut raf::Raf, lang: &CTFLanguage) {
        // Create diag services
        let pool = ECU::read_ecu_pool(reader, &self.diagjob);
        let mut dreader = raf::Raf::from_bytes(&pool, raf::RafByteOrder::LE);
        self.diag_services = (0..self.diagjob.entry_count as usize).map(|diag_job_index| {
            let offset = dreader.read_i32().unwrap();
            let size = dreader.read_i32().unwrap();
            let crc = dreader.read_i32().unwrap();
            let config = dreader.read_i16().unwrap();
            let diag_base_addr = offset + self.diagjob.block_offset;
            DiagService::new(reader, lang, diag_base_addr as i64, diag_job_index as i32, &self)
        }).collect();
    }

    pub fn create_ecu_varients(&mut self, reader: &mut raf::Raf, lang: &CTFLanguage) {
        let var_block = &self.ecuvarient;
        let pool = ECU::read_ecu_pool(reader, &self.ecuvarient);
        let mut vreader = raf::Raf::from_bytes(&pool, raf::RafByteOrder::LE);
        let mut copy = self.clone();
        let res: Vec<ECUVarient> = (0..var_block.entry_count as usize).map(|index|{
            vreader.seek(index * var_block.entry_size as usize);
            let entry_offset = vreader.read_i32().unwrap();
            let entry_size = vreader.read_i32().unwrap();
            let pool_entry_attrib = vreader.read_u16().unwrap();
            let varient_block_address = entry_offset + var_block.block_offset;
            ECUVarient::new(reader, lang,&mut copy, varient_block_address as i64, entry_size)
        }).collect();
        copy.ecu_varient = res;
        *self = copy;

    }

    pub fn read_ecu_pool(reader: &mut raf::Raf, blk: &block) -> Vec<u8> {
        reader.seek(blk.block_offset as usize);
        reader.read_bytes(blk.entry_count as usize * blk.entry_size as usize).expect("Error reading block")
    }
}