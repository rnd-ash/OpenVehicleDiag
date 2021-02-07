pub mod dtc;
pub mod service;
use serde::{Serialize, Deserialize};

/// Derived from the ODX specification
/// See https://www.emotive.de/wiki/index.php?title=Diagnoselayer_und_Diagnosedienste#DATA-OBJECT-PRO.C2.ADPER.C2.ADTY_.28DOP.29

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableData {
    pub name: String,
    pub start: f32,
    pub end: f32
}

impl TableData {
    pub fn get_name(&self) -> String {
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
    /// Value is a hex dump
    HexDump,
    /// Value is an encoded String
    String(StringEncoding),

    // Below lies interpretations for floats and integers
    /// Shortcut version of table, just for 1 == true, 0 == false
    Bool{ pos_name: Option<String>, neg_name: Option<String> },

    /// Coded value is assigned to a text name within a table
    Table(Vec<TableData>),
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

#[cfg(test)]
mod data_test {
    use super::{DataFormat, service::{ParamByteOrder, Parameter}};
    #[test]
    /// Rust a simulation of DT_05 commands found on Delphi CRD ECU (Diag_version_87H) to check parsing
    /// against known values
    pub fn validate_parsing() {
        let req: [u8; 2] = [0x21, 0x05];
        let resp: Vec<u8> = vec![
            0x71, 0x05, 0x00, 0x00, 0x01, 0x18, 0x02, 
            0x6D, 0x00, 0x00, 0x00, 0x08, 0x02, 
            0x80, 0x03, 0xE8, 0x13, 0x88, 0x01, 
            0x2C, 0x00, 0xA0, 0x00, 0x00, 0x00, 
            0xB4, 0x00, 0xB4, 0x00, 0xA0, 0x01,
            0xD0, 0x00, 0x00];

        // DT05_Gearbox Gear - 8
        // DT05_Oil_Temp - -10C
        let gearbox_gear_parser = super::service::Parameter {
            name: "DT_05_Gearbox_Gear".into(),
            unit: "-".into(),
            start_bit: 80,
            length_bits: 16,
            byte_order: ParamByteOrder::BigEndian,
            data_format: DataFormat::Linear{ multiplier: 1.0, offset: 0.0 },
            limits: None
        };

        let oil_temp_parser = super::service::Parameter {
            name: "DT_05_Oil_temperature".into(),
            unit: "°C".into(),
            start_bit: 160,
            length_bits: 16,
            byte_order: ParamByteOrder::BigEndian,
            data_format: DataFormat::Linear{ multiplier: 0.25, offset: -50.0 },
            limits: None
        };

        assert_eq!(gearbox_gear_parser.decode_value_to_number(&resp).unwrap(), 8.0);
        assert_eq!(oil_temp_parser.decode_value_to_number(&resp).unwrap(), -10.0);
    }
}