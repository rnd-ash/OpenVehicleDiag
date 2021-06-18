use core::{num, panic};
use std::{
    borrow::BorrowMut,
    collections::{HashMap, VecDeque},
    convert::TryInto,
    fs::File,
    thread::Result,
    vec,
};

use common::raf::{self, Raf, RafError};

use crate::{machine::{opcode::OP_CODE_LIST, operand::OpAddrMode, operations::read_decrypt_bytes}, virtual_iface::VirtualIface};

use self::{
    flag::Flag,
    opcode::OpCode,
    operand::{Operand, OperandData},
    register::{Register, REGISTER_LIST},
    string_data::StringData,
};

pub(crate) mod arg_info;
pub(crate) mod flag;
mod op_funcs;
pub(crate) mod opcode;
pub(crate) mod operand;
pub(crate) mod operations;
pub(crate) mod register;
pub(crate) mod string_data;

pub const JOB_INIT_NAME: &str = "INITIALISIERUNG";
pub const JOB_NAME_EXIT: &str = "ENDE";
pub const JOB_NAME_IDENT: &str = "IDENTIFIKATION";

pub const BYTE_ARRAY_0: [u8; 1] = [0];

pub type OperationDelegate =
    dyn Fn(&mut Machine, &mut OpCode, &mut Operand, &mut Operand) -> EdiabasResult<()>;
pub type VJobDelegate = fn(m: &mut Machine) -> ();
pub type AbortJobDelegate = fn() -> bool;
pub type ProgressJobDelegate = fn(m: &mut Machine) -> ();
pub type ErrorRaisedDelegate = fn() -> ();

#[derive(Debug, Clone, Default)]
pub struct UsesInfos(Vec<UsesInfo>);

#[derive(Debug, Clone, Default)]
pub struct UsesInfo(String);

#[derive(Debug, Clone, Default)]
pub struct DescriptionInfos {
    pub global_comments: Vec<String>,
    pub job_comments: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum ResultType {
    TypeB, // 8 bit
    TypeW, // 16 bit
    TypeD, // 32 bit
    TypeC, // 8 bit char
    TypeI, // 16 bit signed
    TypeL, // 32 bit signed
    TypeR, // float
    TypeS, // string
    TypeY, // array
}

impl Default for ResultType {
    fn default() -> Self {
        Self::TypeB
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResultData {
    res_type: ResultType,
    name: String,
    op_data: OperandData,
}

#[derive(Debug, Clone, Default)]
pub struct JobInfo {
    pub name: String,
    pub offset: u32,
    pub size: u32,
    pub array_size: u32,
    pub uses_info: Option<UsesInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct JobInfos {
    pub job_info_array: Vec<JobInfo>,
    pub job_name_dict: HashMap<String, u32>,
}

#[derive(Debug, Clone, Default)]
pub struct TableInfo {
    pub name: String,
    pub table_offset: u32,
    pub table_column_offset: u32,
    pub columns: u32,
    pub rows: u32,
    column_name_dict: HashMap<String, u32>,
    seek_column_strings_dict: Vec<HashMap<String, u32>>,
    seek_column_value_dict: Vec<HashMap<u32, u32>>,
    table_entries: Vec<Vec<u32>>,
}

#[derive(Debug, Clone, Default)]
pub struct TableInfos {
    table_info_array: Vec<TableInfo>,
    table_name_dict: HashMap<String, u32>,
}

#[derive(Debug)]
pub enum EdiabasError {
    InvalidDataLength(&'static str, &'static str),
    InvalidAddressMode(&'static str, &'static str),
    InvalidDataType(&'static str, &'static str),
    NullData(&'static str, &'static str),
    OpCodeOutOfRange(&'static str, &'static str),
    OpCodeMappingInvalid(&'static str, &'static str),
    RafError(RafError),
    InvalidSrcDataType(&'static str, &'static str),
    InvalidTargDataType(&'static str, &'static str),
    Todo // Debug only
}

impl From<raf::RafError> for EdiabasError {
    fn from(x: raf::RafError) -> Self {
        EdiabasError::RafError(x)
    }
}

pub type EdiabasResult<T> = std::result::Result<T, EdiabasError>;

#[derive(Clone, Default)]
pub struct Machine {
    pub max_array_size: u32,
    pub table_item_buffer: Vec<u8>,
    pub uses_info: UsesInfos,
    pub job_info: JobInfos,
    pub tables: TableInfos,
    pub req_init: bool,
    pub raf: Option<Raf>,
    disposed: bool,
    job_running: bool,
    job_std: bool,
    job_std_exit: bool,
    close_fs: bool,
    stack: VecDeque<u8>,
    arg_info: arg_info::ArgInfo,
    arg_info_std: arg_info::ArgInfo,
    result_dict: HashMap<String, ResultData>,
    result_sys_dict: HashMap<String, ResultData>,
    result_request_dict: HashMap<String, bool>,
    results_sets: Vec<HashMap<String, ResultData>>,
    results_sets_tmp: Vec<HashMap<String, ResultData>>,
    config_dict: HashMap<String, String>,
    group_mapping_dict: HashMap<String, String>,
    info_progress_range: i64,
    info_progress_pos: i64,
    info_progress_text: String,
    results_job_status: String,
    abort_job_delegate: Option<AbortJobDelegate>,
    progress_func: Option<ProgressJobDelegate>,
    error_delegate: Option<ErrorRaisedDelegate>,
    error_trap_mask: u32,
    error_trap_bit_nr: i32,
    byte_registers: Vec<u8>,
    float_registers: Vec<f32>,
    string_registers: Vec<StringData>,
    flags: Flag,
    pc_counter: u32,
    table_idx: i32,
    table_row_idx: i32,
    token_idx: u32,
    job_end: bool,
    pub iface: VirtualIface
}

impl Machine {
    pub fn new() -> Self {
        let mut ret = Self {
            table_item_buffer: vec![0; 1024],
            error_trap_bit_nr: -1,
            table_idx: -1,
            table_row_idx: -1,
            max_array_size: 1024,
            byte_registers: vec![0; 32],
            float_registers: vec![0f32; 16],
            job_running: false,
            ..Default::default()
        };
        ret.string_registers = vec![StringData::new(ret.max_array_size); 16];
        ret.iface = VirtualIface::new();
        ret
    }

    pub fn load_file(&mut self, f: &mut Raf) {
        if self.raf.is_some() {
            return;
        }
        self.uses_info = self.read_all_uses(f);
        self.job_info = self.read_all_jobs(f);
        self.tables = self.read_all_tables(f);
        self.req_init = true;
        self.raf = Some(f.clone());
        self.exec_job_private(JOB_INIT_NAME, true);
    }

    pub fn get_job_names(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();

        for (job, _) in &self.job_info.job_name_dict {
            if job != JOB_INIT_NAME && job != JOB_NAME_EXIT {
                ret.push(job.clone());
            }
        }
        ret
    }

    pub fn simulate_job(&mut self, name: &str) {
        self.job_end = false;
        self.exec_job_private(name, false);
    }

    fn read_all_uses(&mut self, f: &mut Raf) -> UsesInfos {
        let buffer: Vec<u8>;
        f.seek(0x7C);
        buffer = f.read_bytes(4).expect("Could not read header bytes");
        let uses_offsets = i32::from_le_bytes(buffer[0..4].try_into().unwrap());
        println!("Uses offset: {}", uses_offsets);
        let mut infos = UsesInfos(Vec::new());
        if uses_offsets < 0 {
            return infos;
        }
        f.seek(uses_offsets as usize);
        let uses_count = f.read_i32().unwrap();
        println!("{} use infos", uses_count);

        let mut uses_buffer: [u8; 0x100] = [0; 0x100];
        for _ in 0..uses_count {
            read_decrypt_bytes(f, &mut uses_buffer, 0, 0x100);
            let name = String::from_utf8(Vec::from(uses_buffer))
                .unwrap()
                .trim_matches(char::from(0))
                .to_string();
            infos.0.push(UsesInfo(name))
        }
        infos
    }

    fn read_descriptions(&mut self, f: &mut Raf) -> DescriptionInfos {
        f.seek(0x90);
        let buffer = f.read_bytes(4).unwrap();
        let description_offset = i32::from_le_bytes(buffer[0..4].try_into().unwrap());
        println!("Description offset: {}", description_offset);
        let mut description_info_locals = DescriptionInfos::default();
        if description_offset < 0 {
            return description_info_locals;
        }
        f.seek(description_offset as usize);
        let num_bytes = f.read_i32().unwrap();
        println!("Description byte size: {}", num_bytes);
        let mut comment_list: Vec<String> = Vec::new();
        let mut record_offset = 0;
        let mut record_buffer: [u8; 1100] = [0; 1100];
        let mut previous_job_name: Option<String> = None;
        for _ in 0..num_bytes {
            read_decrypt_bytes(f, &mut record_buffer, record_offset, 1);
            record_offset += 1;
            if record_offset >= 1098 {
                record_offset += 1;
                record_buffer[record_offset as usize] = 10;
            }
            if record_buffer[record_offset as usize - 1] == 10 {
                record_buffer[record_offset as usize] = 0;
                let comment =
                    String::from_utf8(Vec::from(&record_buffer[0..record_offset as usize - 1]))
                        .unwrap();
                if comment.to_ascii_uppercase().starts_with("JOBNAME:") {
                    match &previous_job_name {
                        None => description_info_locals.global_comments = comment_list.clone(),
                        Some(s) => {
                            if !description_info_locals.job_comments.contains_key(s) {
                                description_info_locals
                                    .job_comments
                                    .insert(s.clone(), comment_list.clone());
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
                    description_info_locals
                        .job_comments
                        .insert(s.clone(), comment_list.clone());
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
            return job_list;
        }
        f.seek(job_list_offset as usize);
        let num_jobs = f.read_i32().unwrap();

        let mut job_buffer: [u8; 0x44] = [0; 0x44];
        let mut job_start = f.pos as u32;
        for _ in 0..num_jobs {
            f.seek(job_start as usize);
            let len = job_buffer.len();
            read_decrypt_bytes(f, &mut job_buffer, 0, len);
            let job_name_string = String::from_utf8(Vec::from(&job_buffer[0..0x40]))
                .unwrap()
                .trim_matches(char::from(0))
                .to_string();
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

    pub fn read_all_jobs(&mut self, f: &mut Raf) -> JobInfos {
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
                if key == JOB_INIT_NAME.to_ascii_uppercase()
                    || key == JOB_NAME_EXIT.to_ascii_uppercase()
                {
                    add_job = false;
                }
            }
            if add_job && !job_infos_local.job_name_dict.contains_key(&key) {
                job_infos_local.job_name_dict.insert(key, index as u32);
            }
            job_infos_local.job_info_array[index] = j.clone();
            index += 1;
        }
        return job_infos_local;
    }

    pub fn read_all_tables(&mut self, f: &mut Raf) -> TableInfos {
        f.seek(0x84);
        let mut buffer = f.read_bytes(4).unwrap();
        let table_offset = i32::from_le_bytes(buffer[0..4].try_into().unwrap());
        println!("Table offset: {}", table_offset);
        let mut table_infos = TableInfos::default();

        if table_offset < 0 {
            return table_infos;
        }
        f.seek(table_offset as usize);
        read_decrypt_bytes(f, &mut buffer, 0, 4);
        let table_count = i32::from_le_bytes(buffer[0..4].try_into().unwrap());

        let mut table_start = f.pos;
        for i in 0..table_count {
            let table = self.read_table(f, table_start);
            table_infos.table_info_array.push(table.clone());
            table_infos
                .table_name_dict
                .insert(table.name.to_ascii_uppercase(), i as u32);
            table_start += 0x50;
        }
        println!("Table infos: {:#?}", table_infos);
        table_infos
    }

    fn read_table(&mut self, f: &mut Raf, offset: usize) -> TableInfo {
        f.seek(offset);
        let mut buffer: [u8; 0x50] = [0; 0x50];
        read_decrypt_bytes(f, &mut buffer, 0, 0x50);
        let name = String::from_utf8(Vec::from(&buffer[0..0x40]))
            .unwrap()
            .trim_matches(char::from(0))
            .to_string();
        TableInfo {
            name,
            table_offset: u32::from_le_bytes(buffer[0x40..0x44].try_into().unwrap()),
            table_column_offset: u32::from_le_bytes(buffer[0x44..0x48].try_into().unwrap()),
            columns: u32::from_le_bytes(buffer[0x48..0x4C].try_into().unwrap()),
            rows: u32::from_le_bytes(buffer[0x4C..].try_into().unwrap()),
            ..Default::default()
        }
    }

    fn get_table_index(&mut self, f: &mut Raf, table: &mut TableInfo) {
        if !table.table_entries.is_empty() {
            return;
        }

        f.seek(table.table_column_offset as usize);
        let mut column_name_dict: HashMap<String, u32> = HashMap::new();
        let mut table_entries: Vec<Vec<u32>> = vec![Vec::new(); table.rows as usize + 1];
        for i in 0..table.rows as usize {
            table_entries[i] = vec![0u32; table.columns as usize];
            for x in 0..table.columns as usize {
                table_entries[i][x] = f.pos as u32;
                let mut l = 0;
                for k in 0..self.table_item_buffer.len() {
                    l = k;
                    read_decrypt_bytes(f, &mut self.table_item_buffer, k, 1);
                    if self.table_item_buffer[k] == 0 {
                        break;
                    }
                }
                if i == 0 {
                    let column_name =
                        String::from_utf8(Vec::from(&self.table_item_buffer[0..l])).unwrap();
                    column_name_dict.insert(column_name, i as u32);
                }
            }
        }
        table.column_name_dict = column_name_dict;
        table.table_entries = table_entries;
    }

    fn get_table_string(&mut self, f: &mut Raf, string_offset: u32) -> String {
        f.seek(string_offset as usize);
        let mut l = 0;
        for x in 0..self.table_item_buffer.len() {
            l = x;
            read_decrypt_bytes(f, &mut self.table_item_buffer, x, 1);
            if self.table_item_buffer[x] == 0 {
                break;
            }
        }
        String::from_utf8(Vec::from(&self.table_item_buffer[0..l])).unwrap()
    }

    fn get_table_idx(&mut self, f: &mut Raf, table_idx: u32) -> u32 {
        todo!()
    }

    fn get_job_info(&self, name: &str) -> Option<JobInfo> {
        match self
            .job_info
            .job_name_dict
            .get_key_value(&name.to_ascii_uppercase())
        {
            Some((_, idx)) => Some(self.job_info.job_info_array[*idx as usize].clone()),
            None => None,
        }
    }

    fn exec_job_private(&mut self, name: &str, recursive: bool) {
        // Assumed SGFS is already open
        match self.get_job_info(name) {
            Some(j) => match j.uses_info {
                Some(info) => {
                    todo!(
                        "Cannot run jobs from other PRG files. This job requires {}",
                        info.0
                    )
                }
                None => {
                    let mut tmp = self.raf.as_mut().unwrap().clone();
                    self.exec_job_private_fs(name, recursive, &mut tmp, j)
                }
            },
            None => {
                eprintln!("Warning. Job {} not found!", name);
                return;
            }
        }
    }

    fn exec_job_private_fs(&mut self, name: &str, recursive: bool, fs: &mut Raf, info: JobInfo) {
        println!("Executing JOB {}", name);
        if !self.req_init && !recursive {
            todo!("Request init job");
        }
        let mut buffer: [u8; 2] = [0; 2];
        let mut res_set_tmp: Vec<HashMap<String, ResultData>> = Vec::new();

        self.result_dict.clear();
        self.result_sys_dict.clear();
        self.stack.clear();

        self.string_registers.iter_mut().for_each(|s| {
            s.clear();
        });
        self.error_trap_bit_nr = -1;
        self.error_trap_mask = 0;
        self.info_progress_range = -1;
        self.info_progress_pos = -1;
        self.info_progress_text.clear();
        self.results_job_status.clear();
        self.max_array_size = info.size;
        self.pc_counter = info.offset;

        let mut arg0 = Operand::new(
            OpAddrMode::None,
            OperandData::None,
            OperandData::None,
            OperandData::None,
        );
        let mut arg1 = Operand::new(
            OpAddrMode::None,
            OperandData::None,
            OperandData::None,
            OperandData::None,
        );
        let mut found_first_eoj = false;
        while !self.job_end {
            let pc_counter_old = self.pc_counter;
            fs.seek(self.pc_counter as usize);
            read_decrypt_bytes(fs, &mut buffer, 0, 2);
            let op_code_val = buffer[0];
            let op_addr_mode = buffer[1];

            let op_addr_mode0: OpAddrMode =
                unsafe { ::std::mem::transmute((op_addr_mode & 0xF0) >> 4) };
            let op_addr_mode1: OpAddrMode =
                unsafe { ::std::mem::transmute((op_addr_mode & 0x0F) >> 0) };

            let mut oc = unsafe { OP_CODE_LIST.get(op_code_val as usize) }
                .expect(&format!("No op code for op_code_val {}", op_code_val))
                .clone();

            arg0 = self
                .get_op_arg(fs, op_addr_mode0)
                .expect("Error init op_arg");
            arg1 = self
                .get_op_arg(fs, op_addr_mode1)
                .expect("Error init op_arg");
            self.pc_counter = fs.pos as u32;

            if oc.arg0_is_near_addr && op_addr_mode0 == OpAddrMode::Imm32 {
                let label_addr = self.pc_counter + *arg0.data1.get_integer().unwrap();
                arg0.data1 = OperandData::Integer(label_addr)
            }

            if let Some(del) = &self.abort_job_delegate {
                if del() {
                    panic!("Job aborted!!")
                }
            }

            if let Some(func) = oc.op_func {
                if let Err(e) = func(self, &mut oc, &mut arg0, &mut arg1) {
                    println!("ERROR: {:?}", e);
                    println!(
                        "\n\n--OC NAME {}--\nArg0: {:?}\nArg1: {:?}",
                        oc.pneumonic, arg0, arg1
                    );
                    panic!("Execution failed. Error: {:?}", e);
                }
            }
        }
        if self.result_dict.len() > 0 {
            self.results_sets_tmp.push(self.result_dict.clone());
        }
        println!("Execution of {} - {:?}", name, self.result_dict);
        self.result_dict.clear();

    }

    pub fn get_op_arg(&mut self, fs: &mut Raf, addr_mode: OpAddrMode) -> EdiabasResult<Operand> {
        let mut buffer: [u8; 5] = [0; 5];
        match addr_mode {
            OpAddrMode::None => Ok(Operand::new(
                addr_mode,
                OperandData::None,
                OperandData::None,
                OperandData::None,
            )),
            OpAddrMode::RegS | OpAddrMode::RegAb | OpAddrMode::RegI | OpAddrMode::RegL => {
                read_decrypt_bytes(fs, &mut buffer, 0, 1);
                let oa_reg = Self::get_register(buffer[0])?;
                Ok(Operand::new(
                    addr_mode,
                    OperandData::Register(oa_reg),
                    OperandData::None,
                    OperandData::None,
                ))
            }
            OpAddrMode::Imm8 => {
                read_decrypt_bytes(fs, &mut buffer, 0, 1);
                Ok(Operand::new(
                    addr_mode,
                    OperandData::Integer(buffer[0] as u32),
                    OperandData::None,
                    OperandData::None,
                ))
            }
            OpAddrMode::Imm16 => {
                read_decrypt_bytes(fs, &mut buffer, 0, 2);
                Ok(Operand::new(
                    addr_mode,
                    OperandData::Integer(
                        u16::from_le_bytes(buffer[0..2].try_into().unwrap()) as u32
                    ),
                    OperandData::None,
                    OperandData::None,
                ))
            }
            OpAddrMode::Imm32 => {
                read_decrypt_bytes(fs, &mut buffer, 0, 4);
                Ok(Operand::new(
                    addr_mode,
                    OperandData::Integer(u32::from_le_bytes(buffer[0..4].try_into().unwrap())),
                    OperandData::None,
                    OperandData::None,
                ))
            }
            OpAddrMode::ImmStr => {
                read_decrypt_bytes(fs, &mut buffer, 0, 2);
                let len = u16::from_le_bytes(buffer[0..2].try_into().unwrap());
                let mut buf = vec![0u8; len as usize];
                read_decrypt_bytes(fs, &mut buf, 0, len as usize);
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Bytes(buf), 
                    OperandData::None, 
                    OperandData::None))
            },
            OpAddrMode::IdxImm => {
                read_decrypt_bytes(fs, &mut buffer, 0, 3);
                let oa_reg = Self::get_register(buffer[0])?;
                let idx = u16::from_le_bytes(buffer[1..3].try_into().unwrap());
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg), 
                    OperandData::Integer(idx as u32), 
                    OperandData::None))
            },
            OpAddrMode::IdxReg => {
                read_decrypt_bytes(fs, &mut buffer, 0, 2);
                let oa_reg_0 = Self::get_register(buffer[0])?;
                let oa_reg_1 = Self::get_register(buffer[1])?;
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg_0), 
                    OperandData::Register(oa_reg_1), 
                    OperandData::None))
            },
            OpAddrMode::IdxRegImm => {
                read_decrypt_bytes(fs, &mut buffer, 0, 4);
                let oa_reg_0 = Self::get_register(buffer[0])?;
                let oa_reg_1 = Self::get_register(buffer[1])?;
                let inc = u16::from_le_bytes(buffer[2..4].try_into().unwrap());
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg_0), 
                    OperandData::Register(oa_reg_1), 
                    OperandData::Integer(inc as u32)))
            },
            OpAddrMode::IdxImmLenImm => {
                read_decrypt_bytes(fs, &mut buffer, 0, 5);
                let oa_reg = Self::get_register(buffer[0])?;
                let idx = u16::from_le_bytes(buffer[1..3].try_into().unwrap());
                let len = u16::from_le_bytes(buffer[3..5].try_into().unwrap());
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg), 
                    OperandData::Integer(idx as u32), 
                    OperandData::Integer(len as u32)))
            },
            OpAddrMode::IdxImmLenReg => {
                read_decrypt_bytes(fs, &mut buffer, 0, 4);
                let oa_reg = Self::get_register(buffer[0])?;
                let idx = u16::from_le_bytes(buffer[1..3].try_into().unwrap());
                let len = buffer[3];
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg), 
                    OperandData::Integer(idx as u32), 
                    OperandData::Integer(len as u32)))
            },
            OpAddrMode::IdxRegLenImm => {
                read_decrypt_bytes(fs, &mut buffer, 0, 4);
                let oa_reg = Self::get_register(buffer[0])?;
                let oa_idx = Self::get_register(buffer[1])?;
                let len = u16::from_le_bytes(buffer[2..4].try_into().unwrap());
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg), 
                    OperandData::Register(oa_idx), 
                    OperandData::Integer(len as u32)))
            },
            OpAddrMode::IdxRegLenReg => {
                read_decrypt_bytes(fs, &mut buffer, 0, 3);
                let oa_reg = Self::get_register(buffer[0])?;
                let oa_idx = Self::get_register(buffer[1])?;
                let oa_len = Self::get_register(buffer[2])?;
                Ok(Operand::new(
                    addr_mode, 
                    OperandData::Register(oa_reg), 
                    OperandData::Register(oa_idx), 
                    OperandData::Register(oa_len)))
            },
        }
    }

    pub fn set_error() {}

    pub fn get_register(opcode: u8) -> EdiabasResult<Register> {
        let result: Register;
        if opcode <= 0x33 {
            result = REGISTER_LIST[opcode as usize].clone();
        } else if opcode >= 0x80 {
            let idx: usize = (opcode - 0x80 + 0x34) as usize;
            if idx >= REGISTER_LIST.len() {
                return Err(EdiabasError::OpCodeOutOfRange("mod", "get_register"));
            }
            result = REGISTER_LIST[idx].clone();
        } else {
            return Err(EdiabasError::OpCodeOutOfRange("mod", "get_register"));
        }
        if result.opcode != opcode {
            return Err(EdiabasError::OpCodeMappingInvalid("mod", "get_register"));
        }
        Ok(result)
    }

    pub fn set_result_data(&mut self, data: ResultData) {
        let key = data.name.to_ascii_uppercase();
        self.result_dict.insert(key, data);
    }
}
