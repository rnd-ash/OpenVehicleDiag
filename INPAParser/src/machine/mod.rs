use core::num;
use std::{collections::HashMap, convert::TryInto};

use common::raf::{self, Raf, RafError};

use crate::machine::operations::read_decrypt_bytes;

use self::{opcode::OpCode, operand::Operand, register::Register, string_data::StringData};

pub (crate) mod register;
pub (crate) mod flag;
pub (crate) mod operations;
pub (crate) mod operand;
pub (crate) mod arg_info;
pub (crate) mod opcode;
pub (crate) mod string_data;

pub const JOB_INIT_NAME: &str = "INITIALISIERUNG";
pub const JOB_NAME_EXIT: &str = "ENDE";
pub const JOB_NAME_IDENT: &str = "IDENTIFIKATION";

pub const BYTE_ARRAY_0: [u8; 1] = [0];

pub type OperationDelegate = fn(m: &Machine, oc: &OpCode, arg0: &Operand, arg1: &Operand) -> ();
pub type VJobDelegate = fn(m: &Machine, ) -> ();
pub type AbortJobDelegate = fn() -> bool;
pub type ProgressJobDelegate = fn(m: &Machine) -> ();
pub type ErrorRaisedDelegate = fn() -> ();

#[derive(Debug, Clone, Default)]
pub struct UsesInfos(Vec<UsesInfo>);

#[derive(Debug, Clone, Default)]
pub struct UsesInfo(String);


#[derive(Debug, Clone, Default)]
pub struct DescriptionInfos {
    pub global_comments: Vec<String>,
    pub job_comments: HashMap<String, Vec<String>>
}

#[derive(Debug, Clone, Default)]
pub struct JobInfo {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub array_size: u32,
    pub uses_info: Option<UsesInfo>
}

#[derive(Debug, Clone, Default)]
pub struct JobInfos {
    pub job_info_array: Vec<JobInfo>,
    pub job_name_dict: HashMap<String, u32>
}

pub enum EdiabasError {
    InvalidDataLength,
    InvalidAddressMode,
    InvalidDataType,
    NullData,
    RafError(RafError)
}

impl From<raf::RafError> for EdiabasError {
    fn from(x: raf::RafError) -> Self {
        EdiabasError::RafError(x)
    }
}

pub type Result<T> = std::result::Result<T, EdiabasError>;

#[derive(Debug, Clone, Default)]
pub struct Machine {
    pub float_registers: Vec<Register>,
    pub string_registers: Vec<Register>,
    pub byte_registers: Vec<Register>,
    pub max_array_size: u64
}

impl Machine {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_float_register_by_idx(&self, idx: usize) -> &Register {
        &self.float_registers[idx]
    }

    pub fn get_string_register_by_idx(&self, idx: usize) -> &Register {
        &self.string_registers[idx]
    }

    pub fn get_byte_register_by_idx(&self, idx: usize) -> &Register {
        &self.byte_registers[idx]
    }

    pub fn load_file(&mut self, f: &mut Raf) {
        f.seek(0);
        //self.read_all_uses(f);
        println!("Job infos: {:#?}", self.read_descriptions(f));
        self.read_all_jobs(f);
    }

    fn read_all_uses(&mut self, f: &mut Raf) -> UsesInfos {
        let mut buffer: Vec<u8>;
        f.seek(0x7C);
        buffer = f.read_bytes(4).expect("Could not read header bytes");
        let uses_offsets = i32::from_le_bytes(buffer[0..4].try_into().unwrap());
        println!("Uses offset: {}",uses_offsets);
        let mut infos = UsesInfos(Vec::new());
        if uses_offsets < 0 {
            return infos;
        }
        f.seek(uses_offsets as usize);
        let uses_count = f.read_u32().unwrap();
        println!("{} use infos", uses_count);
        // TODO
        todo!()
    }

    fn read_descriptions(&mut self, f: &mut Raf) -> DescriptionInfos {
        f.seek(0x90);
        let buffer = f.read_bytes(4).unwrap();
        let description_offset = i32::from_le_bytes(buffer[0..4].try_into().unwrap());
        println!("Description offset: {}",description_offset);
        let mut description_info_locals = DescriptionInfos::default();
        if description_offset < 0 {
            return description_info_locals
        }
        f.seek(description_offset as usize);
        let num_bytes = f.read_i32().unwrap();
        println!("Description byte size: {}", num_bytes);
        let mut comment_list: Vec<String> = Vec::new();
        let mut record_offset = 0;
        let mut record_buffer: [u8; 1100] = [0;  1100];
        let mut previous_job_name: Option<String> = None;
        for _ in 0..num_bytes {
            read_decrypt_bytes(f, &mut record_buffer, record_offset, 1);
            record_offset+=1;
            if record_offset >= 1098 {
                record_offset+=1;
                record_buffer[record_offset as usize] = 10;
            }
            if record_buffer[record_offset as usize - 1] == 10 {
                record_buffer[record_offset as usize] = 0;
                let comment = String::from_utf8(Vec::from(&record_buffer[0..record_offset as usize - 1])).unwrap();
                if comment.to_ascii_uppercase().starts_with("JOBNAME:") {
                    match &previous_job_name {
                        None => description_info_locals.global_comments = comment_list.clone(),
                        Some(s) => {
                            if !description_info_locals.job_comments.contains_key(s) {
                                description_info_locals.job_comments.insert(s.clone(), comment_list.clone());
                            }
                        }
                    }
                    comment_list.clear();
                    previous_job_name = Some(comment[0..8].to_string());
                }
                comment_list.push(comment);
                record_offset = 0;
            }
        }
        match &previous_job_name {
            None => description_info_locals.global_comments = comment_list.clone(),
            Some(s) => {
                if !description_info_locals.job_comments.contains_key(s) {
                    description_info_locals.job_comments.insert(s.clone(), comment_list.clone());
                }
            }
        }
        description_info_locals
    }

    fn read_job_list(&mut self, f: &mut Raf, info: Option<UsesInfo>) -> Vec<JobInfo> {
        f.seek(0x18);
        let buffer = f.read_bytes(4).unwrap();
        let mut array_size = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
        if array_size == 0 {
            array_size = 1024;
        }
        f.seek(0x88);
        let buffer = f.read_bytes(4).unwrap();
        let mut job_list_offset = i32::from_le_bytes(buffer[0..4].try_into().unwrap());

        let mut job_list: Vec<JobInfo> = Vec::new();
        if job_list_offset < 0 {
            return job_list
        }
        f.seek(job_list_offset as usize);
        let num_jobs = f.read_i32().unwrap();

        let mut job_buffer: [u8; 0x44] = [0; 0x44];
        let mut job_start = f.pos as u32;
        for _ in 0..num_jobs {
            f.seek(job_start as usize);
            let len = job_buffer.len();
            read_decrypt_bytes(f, &mut job_buffer, 0, len);
            let job_name_string = String::from_utf8(Vec::from(&job_buffer[0..0x40])).unwrap().trim_matches(char::from(0)).to_string();
            let job_address = u32::from_le_bytes(job_buffer[0x40..].try_into().unwrap());
            job_list.push(JobInfo {
                name: job_name_string,
                offset: job_address,
                size: 0,
                array_size,
                uses_info: info.clone(),
            });
            job_start += 0x44;
        }
        job_list
    }

    pub fn read_all_jobs(&mut self, f: &mut Raf) {
        let list = self.read_job_list(f, None);
        // TODO Read jobs from other info files
        let num_jobs = list.len();
        let mut job_infos_local = JobInfos {
            job_info_array: vec![JobInfo::default(); num_jobs],
            job_name_dict: HashMap::new(),
        };
        let mut index = 0usize;
        for j in &list {
            let key = j.name.to_ascii_uppercase();
            let mut add_job = true;
            if j.uses_info.is_some() {
                if key == JOB_INIT_NAME.to_ascii_uppercase() || key == JOB_NAME_EXIT.to_ascii_uppercase() {
                    add_job = false;
                }
            }
            if add_job && !job_infos_local.job_name_dict.contains_key(&key) {
                job_infos_local.job_name_dict.insert(key, index as u32);
            }
            job_infos_local.job_info_array[index] = j.clone();
            index += 1;
        }
        println!("Job list: {:#?}", job_infos_local);
    }
}