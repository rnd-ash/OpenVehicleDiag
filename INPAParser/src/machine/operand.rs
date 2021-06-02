use std::borrow::Borrow;

use super::{EdiabasError, register::Register};


#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpAddrMode {
    None = 0,
    RegS = 1,
    RegAb = 2,
    RegI = 3,
    RegL = 4,
    Imm8 = 5,
    Imm16 = 6,
    Imm32 = 7,
    ImmStr = 8,
    IdxImm = 9,
    IdxReg = 10,
    IdxRegImm = 11,
    IdxImmLenImm = 12,
    IdxImmLenReg = 13,
    IdxRegLenImm = 14,
    IdxRegLenReg = 15,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OperandDataType {
    None,
    Bytes,
    Integer,
    Float,
    Register
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperandData {
    None,
    Bytes(Vec<u8>),
    Integer(u32),
    Float(f32),
    Register(Register)
}

impl OperandData {
    pub fn get_data_type(&self) -> OperandDataType {
        match self {
            &OperandData::None => OperandDataType::None,
            OperandData::Bytes(_) => OperandDataType::Bytes,
            OperandData::Integer(_) => OperandDataType::Integer,
            OperandData::Float(_)=> OperandDataType::Float,
            OperandData::Register(_) => OperandDataType::Register,
        }
    }

    pub fn get_bytes(&self) -> super::Result<&[u8]> {
        if let Self::Bytes(x) = self { Ok(&x) } else { Err(super::EdiabasError::InvalidDataType) }
    }

    pub fn get_integer(&self) -> super::Result<&u32> {
        if let Self::Integer(x) = self { Ok(&x) } else { Err(super::EdiabasError::InvalidDataType) }
    }

    pub fn get_float(&self) -> super::Result<&f32> {
        if let Self::Float(x) = self { Ok(&x) } else { Err(super::EdiabasError::InvalidDataType) }
    }

    pub fn get_register(&self) -> super::Result<&Register> {
        if let Self::Register(x) = self { Ok(&x) } else { Err(super::EdiabasError::InvalidDataType) }
    }
}

#[derive(Debug, Clone)]
pub struct Operand {
    addr_mode: OpAddrMode,
    data1: OperandData,
    data2: OperandData,
    data3: OperandData,
}

impl Operand {
    pub fn new(addr_mode: OpAddrMode, data1: OperandData, data2: OperandData, data3: OperandData) -> Self {
        Self {
            addr_mode,
            data1,
            data2,
            data3
        }
    }

    pub fn get_data_type(&self) -> OperandDataType {
        match self.addr_mode {
            OpAddrMode::RegS |
            OpAddrMode::ImmStr |
            OpAddrMode::IdxImm |
            OpAddrMode::IdxReg |
            OpAddrMode::IdxRegImm |
            OpAddrMode::IdxImmLenImm |
            OpAddrMode::IdxImmLenReg |
            OpAddrMode::IdxRegLenImm |
            OpAddrMode::IdxRegLenReg =>OperandDataType::Bytes,
            _ => OperandDataType::Integer
        }
    }

    pub fn get_value_mask(&self, data_length: u32) -> super::Result<u32> {
        let mut tmp = data_length;
        if data_length == 0 {
            tmp = self.get_data_length(false);
        }
        match tmp {
            1 => Ok(0x000000FF),
            2 => Ok(0x0000FFFF),
            4 => Ok(0xFFFFFFFF),
            _ => Err(super::EdiabasError::InvalidDataLength)
        }
    }

    pub fn get_data_length(&self, write: bool) -> u32 {
        todo!()
    }

    pub fn get_raw_data<T>(&self) -> super::Result<&OperandData> {
        match &self.addr_mode {
            OpAddrMode::RegS |
            OpAddrMode::RegAb |
            OpAddrMode::RegI |
            OpAddrMode::RegL => {
                if self.data1.get_data_type() == OperandDataType::Register { 
                    Ok(&self.data1) 
                } else { 
                    Err(super::EdiabasError::InvalidDataType) 
                }
            }
            OpAddrMode::Imm8 |
            OpAddrMode::Imm16 |
            OpAddrMode::Imm32 |
            OpAddrMode::ImmStr => {
                Ok(&self.data1)
            }
            OpAddrMode::IdxImm |
            OpAddrMode::IdxReg |
            OpAddrMode::IdxRegImm => {
                let arg1_data = self.data1.get_register()?;
            },
            OpAddrMode::IdxImmLenImm => todo!(),
            OpAddrMode::IdxImmLenReg => todo!(),
            OpAddrMode::IdxRegLenImm => todo!(),
            OpAddrMode::IdxRegLenReg => todo!(),
        }
    }

    pub fn get_array_data(&self) -> super::Result<Vec<u8>> {
        todo!()
    }
    
}