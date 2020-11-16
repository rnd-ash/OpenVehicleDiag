use std::num::Wrapping;
use common::raf::{Raf, RafByteOrder};
pub enum StructureName {
    CBFHEADER = 0,
    UNK1 = 1,
    UNK2 = 2,
    UNK3 = 3,
    UNK4 = 4,
    PRESENTATION_STRUCTURE = 0x5,
    UNK6 = 6,
    UNK7 = 7,
    UNK8 = 8,
    UNK9 = 9,
    UNK10 = 0xA,
    SCALEINTERVAL_STRUCTURE = 0xB,
    UNK12 = 0xC,
    UNK13 = 0xD,
    UNK14 = 0xE,
    UNK15 = 0xF,
    FLASH_DESCRIPTION_HEADER = 0x10,
    FLASH_TABLE_STRUCTURE = 0x11,
    UNK18 = 0x12,
    UNK19 = 0x13,
    SESSION_TABLE_STRUCTURE = 0x14,
    UNK21 = 0x15,
    DATA_BLOCK_TABLE_STRUCTURE = 0x16,
    UNK23 = 0x17,
    UNK24 = 0x18,
    UNK25 = 0x19,
    UNK26 = 0x1A,
    SEGMENT_TABLE_STRUCTURE = 0x1B,
    UNK28 = 0x1C,
    CTFHEADER = 0x1D,
    LANGUAGE_TABLE = 0x1E,
    CCFHEADER = 0x1F,
    UNK32 = 0x20,
    CCFFRAGMENT = 0x21,
    UNK34 = 0x22,
    UNK35 = 0x23,
    UNK36 = 0x24,
}

impl StructureName {
    pub fn get_layout(&self) -> Vec<u8> {
        match &self {
            StructureName::CBFHEADER => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK1 => vec![4, 4, 4, 2, 2, 4, 4, 4, 4, 4, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 2, 1],
            StructureName::UNK2 => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2],
            StructureName::UNK3 => vec![6, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 4, 4, 4, 4, 4, 4, 2, 2, 4, 4, 2, 1, 4, 4, 4, 4, 4, 4],
            StructureName::UNK4 => vec![4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 1, 1, 1, 1, 1, 5, 1, 1, 1, 1, 4, 4, 4, 4, 4],
            StructureName::PRESENTATION_STRUCTURE => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 4, 4, 4, 2, 4, 4, 4],
            StructureName::UNK6 => vec![2, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK7 => vec![2, 4, 4, 4, 4, 2, 4, 2, 4],
            StructureName::UNK8 => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2],
            StructureName::UNK9 => vec![2, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK10 => vec![2, 4, 4, 4],
            StructureName::SCALEINTERVAL_STRUCTURE => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK12 => vec![2, 4, 4, 4, 1, 1],
            StructureName::UNK13 => vec![2, 4, 2, 2, 2, 4, 2, 2, 4, 4, 4],
            StructureName::UNK14 => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1],
            StructureName::UNK15 => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1],
            StructureName::FLASH_DESCRIPTION_HEADER => vec![4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::FLASH_TABLE_STRUCTURE => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK18 => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK19 => vec![2, 4, 4, 4, 4],
            StructureName::SESSION_TABLE_STRUCTURE => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2],
            StructureName::UNK21 => vec![2, 4, 4, 4, 4, 4],
            StructureName::DATA_BLOCK_TABLE_STRUCTURE => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK23 => vec![2, 4, 4, 4, 4],
            StructureName::UNK24 => vec![2, 4, 4, 4, 4, 4, 4],
            StructureName::UNK25 => vec![2, 4, 4],
            StructureName::UNK26 => vec![2, 2, 4, 4, 2, 4, 4, 2, 4, 4, 2, 4, 4],
            StructureName::SEGMENT_TABLE_STRUCTURE => vec![2, 4, 4, 4, 4, 4, 4, 4],
            StructureName::UNK28 => vec![2, 4, 4],
            StructureName::CTFHEADER => vec![2, 4, 4, 2, 4, 4, 4, 4, 4],
            StructureName::LANGUAGE_TABLE => vec![2, 4, 2, 4, 4, 4],
            StructureName::CCFHEADER => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2],
            StructureName::UNK32 => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::CCFFRAGMENT => vec![2, 4, 4, 4, 4, 4, 2, 4],
            StructureName::UNK34 => vec![2, 4, 4, 4, 4, 4, 2, 4],
            StructureName::UNK35 => vec![2, 4, 4, 4, 4, 4, 4, 2, 4],
            StructureName::UNK36 => vec![2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 0, 0, 0],
        }
    }
}


pub fn read_cbf_with_offset(memeber_index: i32, name: &StructureName, input: &[u8]) -> i32 {
    let byte_offset = get_cbf_offset(memeber_index, &name, input) as usize;

    let layout = name.get_layout();
    let mut reader  =Raf::from_bytes(&Vec::from(input), RafByteOrder::LE);
    reader.seek(byte_offset);
    match layout[memeber_index as usize] {
        1 => reader.read_i8().unwrap() as i32,
        2 => reader.read_i16().unwrap() as i32,
        4 => reader.read_i32().unwrap() as i32,
        _ => panic!("Cannot read {} bytes as i32!", layout[memeber_index as usize])
    }
}

pub fn read_cbf_with_offset_unsigned(memeber_index: i32, name: &StructureName, input: &[u8]) -> u32 {
    read_cbf_with_offset(memeber_index, name, input) as u32
}


fn get_cbf_offset(memeber_index: i32, name: &StructureName, cbf_input: &[u8]) -> usize {

    let layout = name.get_layout();
    let mut bitmask: Wrapping<u8> = Wrapping(1);
    let mut array_offset = layout[0];
    let mut cbf_offset = 0;

    for i in 0..memeber_index as usize {
        let bitflag_enabled = (bitmask.0 & cbf_input[cbf_offset as usize]) > 0;
        if bitflag_enabled {
            if memeber_index != 1 {
                array_offset = layout[i];
            }
        } else if (memeber_index == 1) && !bitflag_enabled {
            array_offset = 0;
        }

        if bitmask.0 == 0 {
            cbf_offset += 1;
            bitmask.0 = 1;
        } else {
            bitmask *= Wrapping(2);
        }
    }
    array_offset as usize
}