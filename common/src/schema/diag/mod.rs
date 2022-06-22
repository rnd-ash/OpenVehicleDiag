pub mod dtc;
pub mod service;
use std::borrow::Cow;

use serde::{Serialize, Deserialize};

/// Derived from the ODX specification
/// See https://www.emotive.de/wiki/index.php?title=Diagnoselayer_und_Diagnosedienste#DATA-OBJECT-PRO.C2.ADPER.C2.ADTY_.28DOP.29
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableData<'a> {
    pub name: Cow<'a, str>,
    pub start: f32,
    pub end: f32
}

impl<'a> TableData<'a> {
    pub fn get_name(&self) -> Cow<'a, str> {
        self.name.clone()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StringEncoding {
    ASCII,
    Utf8,
    Utf16,
}


pub enum DataDecodeFailure {
    // W_H_A_T  G_O_E_S  H_E_R_E???
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFormat {
    /// Value is encoded as a binary string EG: 0b0001100
    Binary,
    /// Value is a hex dump
    HexDump,
    /// Value is an encoded String
    String(StringEncoding),

    // Below lies interpretations for floats and integers
    /// Shortcut version of table, just for 1 == true, 0 == false
    Bool{ pos_name: Option<Cow<'static, str>>, neg_name: Option<Cow<'static, str>> },

    /// Coded value is assigned to a text name within a table
    Table(Vec<TableData<'static>>),
    /// Coded value is equal to that of the physical value
    Identical,
    /// Coded value is converted from the physical value using a linear function (y=mx+c)
    Linear {
        multiplier: f32,
        offset: f32
    },
    /// Coded value is converted from the physical value section by a linear function
    ScaleLinear,
    /// Coded value is converted from the physical value using a rational function
    RatFunc,
    /// The coded value is converted from the physical value in sections with different rational functions
    ScaleRatFunc,
    /// The coded value is converted from physical value using defined interpolation
    TableInterpretation,
    /// The coded value is converted from the physical value using a Java program with the 'I_CompuCode()' interface
    CompuCode(Vec<u8>) // TODO - How the hell is this supposed to be embedded in JSON!? - Maybe a vector of bytes to send to JVM?
}