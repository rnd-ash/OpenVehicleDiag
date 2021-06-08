use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    fmt::format,
    mem::size_of,
    rc::Weak,
};

use super::{operand::OperandData, EdiabasError, EdiabasResult, Machine};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegisterType {
    RegAb, // 8 bit
    RegI,  // 16 bit
    RegL,  // 32 bit
    RegF,  // float
    RegS,  // String
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegisterDataType {
    Float,
    Integer,
    ByteArray,
    Byte,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegisterData {
    None,
    Float(f32),
    Integer(u32),
    String(super::StringData),
    Bytes(Vec<u8>),
    Byte(u8),
    Short(u16),
}

impl From<OperandData> for RegisterData {
    fn from(x: OperandData) -> Self {
        match x {
            OperandData::None => Self::None,
            OperandData::Bytes(b) => Self::Bytes(b.clone()),
            OperandData::Integer(i) => Self::Integer(i),
            OperandData::Float(f) => Self::Float(f),
            OperandData::String(s) => Self::String(s.clone()),
            OperandData::Register(r) => panic!("Impossible to set register to register data!"),
        }
    }
}

impl RegisterData {
    pub fn get_data_length(&self) -> usize {
        match &self {
            RegisterData::None => 0,
            RegisterData::Float(_) => size_of::<f32>(),
            RegisterData::Integer(_) => 4,
            RegisterData::String(s) => s.get_data_len(),
            RegisterData::Bytes(x) => x.len(),
            RegisterData::Byte(_) => 1,
            RegisterData::Short(_) => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Register {
    pub opcode: u8,
    pub reg_type: RegisterType,
    pub index: usize,
    pub data: RegisterData,
}

impl Register {
    pub const fn new(opcode: u8, reg_type: RegisterType, index: usize) -> Self {
        Self {
            opcode,
            reg_type,
            index,
            data: RegisterData::None,
        }
    }

    pub fn get_name(&self) -> String {
        match self.reg_type {
            RegisterType::RegAb => {
                if self.index > 15 {
                    format!("A{:X}", self.index - 16)
                } else {
                    format!("B{:X}", self.index)
                }
            }
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
            _ => RegisterDataType::Integer,
        }
    }

    pub fn get_value_mask(&self) -> EdiabasResult<u32> {
        match self.get_data_len() {
            1 => Ok(0x000000FF),
            2 => Ok(0x0000FFFF),
            4 => Ok(0xFFFFFFFF),
            _ => Err(super::EdiabasError::InvalidDataLength(
                "register",
                "get_value_mask",
            )),
        }
    }

    pub fn get_data_len(&self) -> u32 {
        self.data.get_data_length() as u32
    }

    pub fn get_raw_data(&self) -> &RegisterData {
        &self.data
    }

    pub fn get_value_data(&self) -> u32 {
        todo!()
    }

    pub fn get_array_data(&self, complete: bool) -> EdiabasResult<Vec<u8>> {
        if self.reg_type != RegisterType::RegS {
            Err(super::EdiabasError::InvalidDataType(
                "register",
                "get_array_data",
            ))
        } else {
            if let RegisterData::String(s) = &self.data {
                Ok(s.get_data(complete))
            } else {
                Err(super::EdiabasError::InvalidDataType(
                    "register",
                    "get_array_data",
                ))
            }
        }
    }

    pub fn set_raw_data(&mut self, v: RegisterData, m: &mut Machine) -> EdiabasResult<()> {
        if self.reg_type == RegisterType::RegS {
            if let RegisterData::Bytes(bytes) = &v {
                self.set_array_data(m, &bytes, false)
            } else {
                Err(EdiabasError::InvalidDataType("register", "set_raw_data"))
            }
        } else if self.reg_type == RegisterType::RegF {
            if let RegisterData::Float(f) = &v {
                self.set_float_data(m, *f)
            } else {
                Err(EdiabasError::InvalidDataType("register", "set_raw_data"))
            }
        } else {
            if let RegisterData::Integer(i) = &v {
                self.set_value_data(m, *i)
            } else {
                Err(EdiabasError::InvalidDataType("register", "set_raw_data"))
            }
        }
    }

    pub fn set_float_data(&mut self, m: &mut Machine, f: f32) -> EdiabasResult<()> {
        if self.reg_type == RegisterType::RegF {
            m.float_registers[self.index] = f;
            Ok(())
        } else {
            Err(EdiabasError::InvalidDataType("register", "set_float_data"))
        }
    }

    /// keep_length default: false
    pub fn set_array_data(
        &mut self,
        m: &mut Machine,
        a: &[u8],
        keep_length: bool,
    ) -> EdiabasResult<()> {
        if self.reg_type == RegisterType::RegS {
            {
                let mut reg = m.string_registers[self.index].borrow_mut().clone();
                reg.set_data(m, a, keep_length);
                m.string_registers[self.index] = reg;
            }
            Ok(())
        } else {
            Err(EdiabasError::InvalidDataType("register", "set_array_data"))
        }
    }

    pub fn clear_data(&mut self, m: &mut Machine) -> EdiabasResult<()> {
        if self.reg_type == RegisterType::RegS {
            m.string_registers[self.index].clear();
            Ok(())
        } else {
            Err(EdiabasError::InvalidDataType("register", "clear_data"))
        }
    }

    pub fn set_value_data(&mut self, m: &mut Machine, data: u32) -> EdiabasResult<()> {
        match self.reg_type {
            RegisterType::RegAb => {
                m.byte_registers[self.index] = data as u8;
                Ok(())
            }
            RegisterType::RegI => {
                let offset = self.index << 1;
                m.byte_registers[offset] = data as u8;
                m.byte_registers[offset + 1] = (data >> 8) as u8;
                Ok(())
            }
            RegisterType::RegL => {
                let offset = self.index << 2;
                m.byte_registers[offset] = data as u8;
                m.byte_registers[offset + 1] = (data >> 8) as u8;
                m.byte_registers[offset + 2] = (data >> 16) as u8;
                m.byte_registers[offset + 3] = (data >> 24) as u8;
                Ok(())
            }
            _ => Err(EdiabasError::InvalidDataType("register", "set_value_data")),
        }
    }
}

pub const REGISTER_LIST: &'static [Register] = &[
    Register::new(0x00, RegisterType::RegAb, 0),
    Register::new(0x01, RegisterType::RegAb, 1),
    Register::new(0x02, RegisterType::RegAb, 2),
    Register::new(0x03, RegisterType::RegAb, 3),
    Register::new(0x04, RegisterType::RegAb, 4),
    Register::new(0x05, RegisterType::RegAb, 5),
    Register::new(0x06, RegisterType::RegAb, 6),
    Register::new(0x07, RegisterType::RegAb, 7),
    Register::new(0x08, RegisterType::RegAb, 8),
    Register::new(0x09, RegisterType::RegAb, 9),
    Register::new(0x0A, RegisterType::RegAb, 10),
    Register::new(0x0B, RegisterType::RegAb, 11),
    Register::new(0x0C, RegisterType::RegAb, 12),
    Register::new(0x0D, RegisterType::RegAb, 13),
    Register::new(0x0E, RegisterType::RegAb, 14),
    Register::new(0x0F, RegisterType::RegAb, 15),
    Register::new(0x10, RegisterType::RegI, 0),
    Register::new(0x11, RegisterType::RegI, 1),
    Register::new(0x12, RegisterType::RegI, 2),
    Register::new(0x13, RegisterType::RegI, 3),
    Register::new(0x14, RegisterType::RegI, 4),
    Register::new(0x15, RegisterType::RegI, 5),
    Register::new(0x16, RegisterType::RegI, 6),
    Register::new(0x17, RegisterType::RegI, 7),
    Register::new(0x18, RegisterType::RegL, 0),
    Register::new(0x19, RegisterType::RegL, 1),
    Register::new(0x1A, RegisterType::RegL, 2),
    Register::new(0x1B, RegisterType::RegL, 3),
    Register::new(0x1C, RegisterType::RegS, 0),
    Register::new(0x1D, RegisterType::RegS, 1),
    Register::new(0x1E, RegisterType::RegS, 2),
    Register::new(0x1F, RegisterType::RegS, 3),
    Register::new(0x20, RegisterType::RegS, 4),
    Register::new(0x21, RegisterType::RegS, 5),
    Register::new(0x22, RegisterType::RegS, 6),
    Register::new(0x23, RegisterType::RegS, 7),
    Register::new(0x24, RegisterType::RegF, 0),
    Register::new(0x25, RegisterType::RegF, 1),
    Register::new(0x26, RegisterType::RegF, 2),
    Register::new(0x27, RegisterType::RegF, 3),
    Register::new(0x28, RegisterType::RegF, 4),
    Register::new(0x29, RegisterType::RegF, 5),
    Register::new(0x2A, RegisterType::RegF, 6),
    Register::new(0x2B, RegisterType::RegF, 7),
    Register::new(0x2C, RegisterType::RegS, 8),
    Register::new(0x2D, RegisterType::RegS, 9),
    Register::new(0x2E, RegisterType::RegS, 10),
    Register::new(0x2F, RegisterType::RegS, 11),
    Register::new(0x30, RegisterType::RegS, 12),
    Register::new(0x31, RegisterType::RegS, 13),
    Register::new(0x32, RegisterType::RegS, 14),
    Register::new(0x33, RegisterType::RegS, 15),
    Register::new(0x80, RegisterType::RegAb, 16),
    Register::new(0x81, RegisterType::RegAb, 17),
    Register::new(0x82, RegisterType::RegAb, 18),
    Register::new(0x83, RegisterType::RegAb, 19),
    Register::new(0x84, RegisterType::RegAb, 20),
    Register::new(0x85, RegisterType::RegAb, 21),
    Register::new(0x86, RegisterType::RegAb, 22),
    Register::new(0x87, RegisterType::RegAb, 23),
    Register::new(0x88, RegisterType::RegAb, 24),
    Register::new(0x89, RegisterType::RegAb, 25),
    Register::new(0x8A, RegisterType::RegAb, 26),
    Register::new(0x8B, RegisterType::RegAb, 27),
    Register::new(0x8C, RegisterType::RegAb, 28),
    Register::new(0x8D, RegisterType::RegAb, 29),
    Register::new(0x8E, RegisterType::RegAb, 30),
    Register::new(0x8F, RegisterType::RegAb, 31),
    Register::new(0x90, RegisterType::RegI, 8),
    Register::new(0x91, RegisterType::RegI, 9),
    Register::new(0x92, RegisterType::RegI, 10),
    Register::new(0x93, RegisterType::RegI, 11),
    Register::new(0x94, RegisterType::RegI, 12),
    Register::new(0x95, RegisterType::RegI, 13),
    Register::new(0x96, RegisterType::RegI, 14),
    Register::new(0x97, RegisterType::RegI, 15),
    Register::new(0x98, RegisterType::RegL, 4),
    Register::new(0x99, RegisterType::RegL, 5),
    Register::new(0x9A, RegisterType::RegL, 6),
    Register::new(0x9B, RegisterType::RegL, 7),
];
