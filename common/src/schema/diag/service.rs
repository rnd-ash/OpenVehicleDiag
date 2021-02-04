use std::{cmp::min, string::FromUtf8Error};
use bit_field::BitArray;
use serde::{Serialize, Deserialize};
use super::DataFormat;
use serde_with::{serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    pub name: String,
    pub description: String,
    #[serde_as(as = "serde_with::hex::Hex<serde_with::formats::Uppercase>")]
    pub payload: Vec<u8>,
    pub input_params: Vec<Parameter>,
    pub output_params: Vec<Parameter>
}

impl Service {
    pub fn service_has_input(&self) -> bool {
        !self.input_params.is_empty()
    }

    pub fn service_has_output(&self) -> bool {
        !self.output_params.is_empty()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamByteOrder {
    BigEndian,
    LittleEndian
}

#[derive(Debug)]
pub enum ParamDecodeError {
    NotImplemented,
    BitRangeError,
    DecodeNotSupported,
    StringDecodeFailure(FromUtf8Error)
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub unit: String,
    pub start_bit: usize,
    pub length_bits: usize,
    pub byte_order: ParamByteOrder,
    pub data_format: DataFormat
}

impl Parameter {
    pub fn decode_value_to_string(&self, input: &[u8]) -> std::result::Result<String, ParamDecodeError> {
        let mut result: String = String::new();
        match &self.data_format {
            DataFormat::RawInt => result.push_str(format!("{}", self.get_number(input)? as u32).as_str()),
            DataFormat::RawFloat => result.push_str(format!("{}", self.get_number(input)? as f32).as_str()),
            DataFormat::HexDump => {
                let start_byte = self.start_bit/8;
                let end_byte = (self.start_bit+self.length_bits)/8;
                return Ok(format!("{:02X?}", &input[start_byte..end_byte]))
            }
            DataFormat::String(_s) => { // TODO take into account encoding of string
                let start_byte = self.start_bit/8;
                let end_byte = (self.start_bit+self.length_bits)/8;
                return match String::from_utf8(Vec::from(&input[start_byte..end_byte])) {
                    Ok(s) => Ok(s),
                    Err(e) => Err(ParamDecodeError::StringDecodeFailure(e)),
                }
            }
            DataFormat::Bool { pos_name, neg_name } => {
                return match self.get_number(input)? {
                    0 => Ok(neg_name.clone().unwrap_or("False".into())),
                    _ => Ok(pos_name.clone().unwrap_or("True".into()))
                }
            }
            DataFormat::Table(t) => {
                let raw = self.get_number(input)? as f32;
                for v in t {
                    if v.start>= raw && v.end <= raw {
                        return Ok(v.name.clone());
                    }
                }
                // Our value wasn't found, undefined value?
                return Ok(format!("Undefined ({})", raw));
            }
            DataFormat::Identical => result.push_str(format!("{}", self.get_number(input)? as f32).as_str()),
            DataFormat::Linear { multiplier, offset } => {
                let res = self.get_number(input)? as f32;
                result.push_str(format!("{}", (res*multiplier) + offset).as_str())
            },
            DataFormat::ScaleLinear => return Err(ParamDecodeError::NotImplemented),
            DataFormat::RatFunc => return Err(ParamDecodeError::NotImplemented),
            DataFormat::ScaleRatFunc => return Err(ParamDecodeError::NotImplemented),
            DataFormat::TableInterpretation => return Err(ParamDecodeError::NotImplemented),
            DataFormat::CompuCode(_) => return Err(ParamDecodeError::NotImplemented)
        }
        // For numbers
        if let Some(end) = self.get_unit() {
            result.push(' ');
            result.push_str(end.as_str());
        }
        Ok(result)
    }

    pub fn decode_value_to_number(&self, input: &[u8]) -> std::result::Result<f32, ParamDecodeError> {
        match &self.data_format {
            DataFormat::RawInt => Ok(self.get_number(input)? as f32),
            DataFormat::RawFloat => Ok(self.get_number(input)? as f32),
            DataFormat::HexDump => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::String(_) => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::Bool { pos_name: _, neg_name: _ } => Ok(self.get_number(input)? as f32),
            DataFormat::Table(_) => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::Identical => Err(ParamDecodeError::NotImplemented),
            DataFormat::Linear { multiplier, offset } => Ok((self.get_number(input)? as f32 * multiplier) + offset),
            DataFormat::ScaleLinear => Err(ParamDecodeError::NotImplemented),
            DataFormat::RatFunc => Err(ParamDecodeError::NotImplemented),
            DataFormat::ScaleRatFunc => Err(ParamDecodeError::NotImplemented),
            DataFormat::TableInterpretation => Err(ParamDecodeError::NotImplemented),
            DataFormat::CompuCode(_) => Err(ParamDecodeError::NotImplemented)
        }
    }

    /// Returns if the data type is capable of being plotted on a chart or not
    pub fn can_plot(&self) -> bool {
        match &self.data_format {
            DataFormat::RawInt => true,
            DataFormat::RawFloat => true,
            DataFormat::HexDump => false,
            DataFormat::String(_) => false,
            DataFormat::Bool { pos_name: _, neg_name: _ } => true,
            DataFormat::Table(_) => false,
            DataFormat::Identical => true,
            DataFormat::Linear { multiplier: _, offset: _ } => true,
            DataFormat::ScaleLinear => false,
            DataFormat::RatFunc => false,
            DataFormat::ScaleRatFunc => false,
            DataFormat::TableInterpretation => false,
            DataFormat::CompuCode(_) => false
        }
    }


    pub fn get_unit(&self) -> Option<String> {
        if self.unit.is_empty() {
            None
        } else {
            Some(self.unit.clone())
        }
    }

    fn get_number(&self, resp: &[u8]) -> std::result::Result<u32, ParamDecodeError> {
        if self.length_bits <= 32 {
            let result = std::panic::catch_unwind(||{
                let mut tmp_start = self.start_bit;
                if self.length_bits <= 8 {
                    resp.get_bits(self.start_bit..self.start_bit+self.length_bits) as u32
                } else {
                    let mut res = 0;
                    if self.byte_order == ParamByteOrder::LittleEndian {
                        // Decode LE
                        let mut shift: usize = 0;
                        while tmp_start < self.start_bit+self.length_bits {
                            let max_read = min(8, (self.start_bit+self.length_bits)-tmp_start);
                            res |= (resp.get_bits(tmp_start..tmp_start+max_read) as u32) << shift;
                            tmp_start += 8;
                            shift += 8;
                        }
                    } else {
                        // Decode BE
                        let mut shift: usize = (self.start_bit+self.length_bits)-8;
                        while tmp_start < self.start_bit+self.length_bits {
                            let max_read = min(8, (self.start_bit+self.length_bits)-tmp_start);
                            res |= (resp.get_bits(tmp_start..tmp_start+max_read) as u32) << shift;
                            tmp_start += 8;
                            shift -= 8;
                        }
                    }
                    res as u32
                }
            });

            match result {
                Ok(r) => Ok(r as u32),
                Err(_) => Err(ParamDecodeError::BitRangeError)
            }
        } else {
            Err(ParamDecodeError::BitRangeError)
        }
    }
}