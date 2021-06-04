use std::borrow::{Borrow, Cow};

use super::{EdiabasError, Machine, register::Register};


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

    pub fn get_value_mask(&self, m: &Machine, data_length: u32) -> super::Result<u32> {
        let mut tmp = data_length;
        if data_length == 0 {
            tmp = self.get_data_length(m,false)?;
        }
        match tmp {
            1 => Ok(0x000000FF),
            2 => Ok(0x0000FFFF),
            4 => Ok(0xFFFFFFFF),
            _ => Err(super::EdiabasError::InvalidDataLength)
        }
    }

    pub fn get_data_length(&self, m: &Machine, write: bool) -> super::Result<u32> {
        match self.addr_mode {
            OpAddrMode::RegS |
            OpAddrMode::ImmStr => Ok(self.get_array_data(m)?.len() as u32),
            OpAddrMode::RegAb => Ok(1),
            OpAddrMode::RegI => Ok(2),
            OpAddrMode::RegL => Ok(4),
            OpAddrMode::Imm8 => Ok(1),
            OpAddrMode::Imm16 => Ok(2),
            OpAddrMode::Imm32 => Ok(4),
            OpAddrMode::IdxImm |
            OpAddrMode::IdxReg |
            OpAddrMode::IdxRegImm => if write {Ok(1)} else { Ok(self.get_array_data(m)?.len() as u32) },
            OpAddrMode::IdxImmLenImm |
            OpAddrMode::IdxImmLenReg |
            OpAddrMode::IdxRegLenImm |
            OpAddrMode::IdxRegLenReg => Ok(self.get_array_data(m)?.len() as u32),
            _ => Ok(0)
        }
    }

    pub fn get_raw_data(&self, m: &super::Machine) -> super::Result<OperandData> {
        match &self.addr_mode {
            OpAddrMode::RegS |
            OpAddrMode::RegAb |
            OpAddrMode::RegI |
            OpAddrMode::RegL => {
                if self.data1.get_data_type() == OperandDataType::Register { 
                    Ok(self.data1.clone()) 
                } else { 
                    Err(super::EdiabasError::InvalidDataType) 
                }
            }
            OpAddrMode::Imm8 |
            OpAddrMode::Imm16 |
            OpAddrMode::Imm32 |
            OpAddrMode::ImmStr => {
                Ok(self.data1.clone())
            }
            OpAddrMode::IdxImm |
            OpAddrMode::IdxReg |
            OpAddrMode::IdxRegImm => {
                let arg1_data = self.data1.get_register()?;
                let data_array = arg1_data.get_array_data(true)?;
                let mut index: u32;
                if self.addr_mode == OpAddrMode::IdxImm {
                    index = *self.data2.get_integer()?;
                } else {
                    index = self.data2.get_register()?.get_value_data();
                }

                if self.addr_mode == OpAddrMode::IdxRegLenImm {
                    index += * self.data3.get_integer()?;
                }
                let required_length: u64 = index as u64 + 1;
                if required_length > m.max_array_size {
                    // TODO SetError
                    return Ok(OperandData::Bytes(Vec::from(super::BYTE_ARRAY_0)))
                }
                if data_array.len() < required_length as usize {
                    return Ok(OperandData::Bytes(Vec::from(super::BYTE_ARRAY_0)))
                }
                let res = Vec::from(&data_array[index as usize..data_array.len()]);
                Ok(OperandData::Bytes(res))
            },
            OpAddrMode::IdxImmLenImm |
            OpAddrMode::IdxImmLenReg |
            OpAddrMode::IdxRegLenImm |
            OpAddrMode::IdxRegLenReg => {
                let data_array = self.data1.get_register()?.get_array_data(true)?;
                let mut index = if self.addr_mode == OpAddrMode::IdxImmLenImm || self.addr_mode == OpAddrMode::IdxImmLenReg {
                    *self.data2.get_integer()?
                } else {
                    self.data2.get_register()?.get_value_data()
                };
                let mut len = if self.addr_mode == OpAddrMode::IdxImmLenImm || self.addr_mode == OpAddrMode::IdxImmLenReg {
                    *self.data3.get_integer()?
                } else {
                    self.data3.get_register()?.get_value_data()
                };
                let required_length: u64 = index as u64 + len as u64;
                if required_length > m.max_array_size {
                    // TODO SetError
                    return Ok(OperandData::Bytes(Vec::from(super::BYTE_ARRAY_0)))
                }
                if data_array.len() < required_length as usize {
                    if data_array.len() <= index as usize {
                        return Ok(OperandData::Bytes(Vec::from(super::BYTE_ARRAY_0)))
                    }
                    len = data_array.len() as u32 - index;
                }
                let res = Vec::from(&data_array[index as usize..len as usize]);
                Ok(OperandData::Bytes(res))

            },
            _ => Err(super::EdiabasError::InvalidAddressMode)
        }
    }


    pub fn get_value_data(&self, len: u32, m: &super::Machine) -> super::Result<u32> {
        match self.get_raw_data(m)? {
            OperandData::Bytes(b) => {
                if len == 0 {
                    return Err(super::EdiabasError::InvalidDataLength)
                }
                let mut bytes = b;
                if bytes.len() < len as usize {
                    for _ in 0..bytes.len() - len as usize {
                        bytes.push(0);
                    }
                }
                let mut value: u32 = 0;
                for i in (0..=len-1).rev() {
                    value <<= 8;
                    value |= bytes[i as usize] as u32;
                }
                Ok(value)
            },
            OperandData::Integer(i) => Ok(i & self.get_value_mask(m,0)?),
            _ => Err(super::EdiabasError::InvalidDataType)
        }
    }

    pub fn get_float_data(&self, m: &Machine) -> super::Result<f32> {
        Ok(*self.get_raw_data(m)?.get_float()?)
    }

    pub fn get_array_data(&self, m: &Machine) -> super::Result<Vec<u8>> {
        Ok(Vec::from(self.get_raw_data(m)?.get_bytes()?))
    }

    pub fn get_string_data(&self, m: &Machine) -> super::Result<Cow<str>> {
        let data = self.get_array_data(m)?;
        todo!();
        //Ok(String::from_utf8_lossy(&data).clone())
    }
    
}