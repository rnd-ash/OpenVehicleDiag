use std::{cmp::min, collections::VecDeque, string::FromUtf8Error};
use bit_field::BitArray;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use serde::{Serialize, Deserialize};
use super::{DataFormat, StringEncoding};
use serde_with::{serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Service {
    pub name: String,
    pub description: String,
    #[serde_as(as = "serde_with::hex::Hex<serde_with::formats::Uppercase>")]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
    pub input_params: Vec<Parameter>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::new")]
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
pub struct Limit {
    upper: f32,
    lower: f32,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub unit: String,
    pub start_bit: usize,
    pub length_bits: usize,
    pub byte_order: ParamByteOrder,
    pub data_format: DataFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "Option::default")]
    pub valid_bounds: Option<Limit>,
}

impl Parameter {
    pub fn decode_value_to_string(&self, input: &[u8]) -> std::result::Result<String, ParamDecodeError> {
        let mut result: String = String::new();
        match &self.data_format {
            DataFormat::HexDump => {
                let start_byte = self.start_bit/8;
                let end_byte = (self.start_bit+self.length_bits)/8;
                return Ok(format!("{:02X?}", &input[start_byte..min(end_byte, input.len())]))
            }
            DataFormat::String(s) => { 
                let start_byte = self.start_bit/8;
                let mut end_byte = (self.start_bit+self.length_bits)/8;
                if *s == StringEncoding::Utf16  {
                    if (start_byte-end_byte) % 2 !=  0 {
                        end_byte += 1;
                    }
                    // Convert out byte array to u16 array
                    let mut dst: Vec<u16> = Vec::with_capacity((start_byte-end_byte)/2);
                    for i in (start_byte..end_byte).step_by(2) {
                        match self.byte_order {
                            ParamByteOrder::BigEndian => { dst.push((input[i] as u16) << 8 | input[i+1] as u16) }
                            ParamByteOrder::LittleEndian => { dst.push((input[i+1] as u16) << 8 | input[i] as u16) }
                        }
                    }
                    return Ok(String::from_utf16_lossy(&dst).to_string())
                } else { // Utf8 can handle ASCII Strings
                    return Ok(String::from_utf8_lossy(&input[start_byte..end_byte]).to_string())
                }
                
            }
            DataFormat::Bool { pos_name, neg_name } => {
                return match self.get_number(input)? {
                    0 => Ok(neg_name.clone().unwrap_or("False".into())),
                    _ => Ok(pos_name.clone().unwrap_or("True".into()))
                }
            }
            DataFormat::Binary => {
                let start_byte = self.start_bit/8;
                let end_byte = (self.start_bit+self.length_bits)/8;
                if start_byte == end_byte { // 1 binary value
                    return Ok(format!("b{:08b}", &input[start_byte]))
                } else { // Multiple binary values!
                    let mut res = String::from("[");
                    for i in &input[start_byte..min(end_byte, input.len())] {
                        res.push_str(&format!("b{:08b}, ", i));
                    }
                    res.drain(res.len()-2..res.len());
                    res.push(']');
                    return Ok(res);
                }
            },
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
            DataFormat::HexDump => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::Binary => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::String(_) => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::Bool { pos_name: _, neg_name: _ } => Ok(if self.get_number(input)? as f32 > 0.0 { 1.0 }  else { 0.0 }),
            DataFormat::Table(_) => Err(ParamDecodeError::DecodeNotSupported),
            DataFormat::Identical => Ok(self.get_number(input)? as f32),
            DataFormat::Linear { multiplier, offset } => Ok((self.get_number(input)? as f32 * multiplier) + offset),
            DataFormat::ScaleLinear => Err(ParamDecodeError::NotImplemented),
            DataFormat::RatFunc => Err(ParamDecodeError::NotImplemented),
            DataFormat::ScaleRatFunc => Err(ParamDecodeError::NotImplemented),
            DataFormat::TableInterpretation => Err(ParamDecodeError::NotImplemented),
            DataFormat::CompuCode(_) => Err(ParamDecodeError::NotImplemented),
        }
    }

    /// Returns if the data type is capable of being plotted on a chart or not
    pub fn can_plot(&self) -> bool {
        match &self.data_format {
            DataFormat::HexDump => false,
            DataFormat::Binary => false,
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
                if self.length_bits <= 8 {
                    resp.get_bits(self.start_bit..self.start_bit+self.length_bits) as u32
                } else {
                    let mut res = 0;
                    let mut buf: Vec<u8> = Vec::new();
                    let mut start = self.start_bit;
                    while start < self.length_bits + self.start_bit {
                        let max_read = min(self.start_bit + self.length_bits, start + 8);
                        buf.push(resp.get_bits(start..max_read));
                        start += 8;
                    }
                    
                    if buf.len() > 4 {
                        panic!("Number too big!") // Cannot handle more than 32bits atm
                    } else {
                        if buf.len() >= 4 {                        
                            res = match self.byte_order {
                                ParamByteOrder::BigEndian => BigEndian::read_u32(&buf),
                                ParamByteOrder::LittleEndian => LittleEndian::read_u32(&buf)
                            }
                        } else if buf.len() >= 2 {
                            res = match self.byte_order {
                                ParamByteOrder::BigEndian => BigEndian::read_u16(&buf) as u32,
                                ParamByteOrder::LittleEndian => LittleEndian::read_u16(&buf) as u32
                            }
                        }
                        res as u32
                    }
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