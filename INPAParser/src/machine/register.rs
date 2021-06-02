use std::{fmt::format, mem::size_of};



#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegisterType {
    RegAb, // 8 bit
    RegI,  // 16 bit
    RegL,  // 32 bit
    RegF,  // float
    RegS   // String
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegisterDataType {
    Float,
    Integer,
    ByteArray,
    Byte
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterData {
    Float(f32),
    Integer(u32),
    String(String),
    Bytes(Vec<u8>),
    Byte(u8),
    Short(u16)
}

impl RegisterData {
    pub fn get_data_length(&self) -> usize {
        match &self {
            RegisterData::Float(_) => size_of::<f32>(),
            RegisterData::Integer(_) => 4,
            RegisterData::String(s) => s.len(),
            RegisterData::Bytes(x) => x.len(),
            RegisterData::Byte(_) => 1,
            RegisterData::Short(_) => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Register<'a> {
    machine: &'a super::Machine,
    opcode: u8,
    reg_type: RegisterType,
    index: usize,
    data: RegisterData
}

impl<'a> Register<'a> {
    pub fn get_name(&self) -> String {
        match self.reg_type {
            RegisterType::RegAb => {
                if self.index > 15 {
                    format!("A{:X}", self.index-16)
                } else {
                    format!("B{:X}", self.index)
                }
            },
            RegisterType::RegI => format!("I{:X}", self.index),
            RegisterType::RegL => format!("L{:X}", self.index),
            RegisterType::RegF => format!("F{:X}", self.index),
            RegisterType::RegS => format!("S{:X}", self.index),
        }
    }

    pub fn get_data_type(&self) -> RegisterDataType {
        match self.reg_type {
            RegisterType::RegF => RegisterDataType::Float,
            RegisterType::RegS => RegisterDataType::ByteArray,
            _ => RegisterDataType::Integer
        }
    }

    pub fn get_value_mask(&self) -> super::Result<u32> {
        match self.get_data_len() {
            1 => Ok(0x000000FF),
            2 => Ok(0x0000FFFF),
            4 => Ok(0xFFFFFFFF),
            _ => Err(super::EdiabasError::InvalidDataLength)
        }
    }

    pub fn get_data_len(&self) -> u32 {
        self.data.get_data_length() as u32
    }

    pub fn get_raw_data(&self) -> &RegisterData {
        &self.data
    }

    pub fn get_array_data(&self, complete: bool) -> super::Result<Vec<u8>> {
        if self.reg_type != RegisterType::RegS {
            Err(super::EdiabasError::InvalidDataType)
        } else {
            Ok(self.machine.string_registers[self.index].get_data(complete))
        }
    }
}