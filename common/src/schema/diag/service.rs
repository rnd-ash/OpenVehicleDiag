use serde::{Serialize, Deserialize};

#[macro_use]
use serde_with::{serde_as};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DataType {
    None,
    String,
    Hex,
    Binary,
    Int,
    Float,
    Enum
}

pub enum DataInput {
    None,
    String(String),
    Hex(Vec<u8>),
    Binary(Vec<bool>),
    Int(u32),
    Float(f32),
    Enum(u32)
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub description: String,
    pub input_type: DataType,
    #[serde_as(as = "serde_with::hex::Hex<serde_with::formats::Uppercase>")]
    pub payload: Vec<u8>,
    pub input_params: Vec<Parameter>,
    pub output_params: Vec<Parameter>
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub start_bit: usize,
    pub length_bits: usize,
    #[serde_as(as = "serde_with::hex::Hex<serde_with::formats::Uppercase>")]
    pub dump: Vec<u8>,
    pub data_type: DataType
}