use std::vec;
use common::raf::{self};

pub mod creader;
pub mod container;


#[derive(Debug)]
pub enum CaesarError {
    FileError(raf::RafError),
    ProcessException(String)
}

impl From<raf::RafError> for CaesarError {
    fn from(x: raf::RafError) -> Self {
        Self::FileError(x)
    }
}

pub type Result<T> = std::result::Result<T, CaesarError>;

/// Based on reverse engineering of c32s.dll


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum StructureName {
    CBF_HEADER = 0,
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
    SCALE_INTERVAL_STRUCTURE = 0xB,
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
    CTF_HEADER = 0x1D,
    LANGUAGE_TABLE = 0x1E,
    CCF_HEADER = 0x1F,
    UNK32 = 0x20,
    CCF_FRAGMENT = 0x21,
    UNK34 = 0x22,
    UNK35 = 0x23,
    UNK36 = 0x24,
}


impl StructureName {
    pub (crate) fn get_layout(&self) -> Vec<u8> {
        match &self {
            StructureName::CBF_HEADER => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
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
            StructureName::SCALE_INTERVAL_STRUCTURE => vec![2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
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
            StructureName::CTF_HEADER => vec![2, 4, 4, 2, 4, 4, 4, 4, 4],
            StructureName::LANGUAGE_TABLE => vec![2, 4, 2, 4, 4, 4],
            StructureName::CCF_HEADER => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2],
            StructureName::UNK32 => vec![6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4],
            StructureName::CCF_FRAGMENT => vec![2, 4, 4, 4, 4, 4, 2, 4],
            StructureName::UNK34 => vec![2, 4, 4, 4, 4, 4, 2, 4],
            StructureName::UNK35 => vec![2, 4, 4, 4, 4, 4, 4, 2, 4],
            StructureName::UNK36 => vec![2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 0, 0, 0],
        }
    }
}

/// Checks if a bitflag is active and returns the offset in the structure
pub fn get_cbf_offset(member_idx: usize, structure: StructureName, cbf_input: &[u8]) -> usize {
    let structure_layout = structure.get_layout();
    let mut bit_mask = 1;
    let mut array_offset = structure_layout[0];
    let mut cbf_offset = 0;

    for i in 0..member_idx {
        let bitflag_enabled = (bit_mask & cbf_input[cbf_offset]) > 0;
        if bitflag_enabled {
            if member_idx != i {
                array_offset += structure_layout[i];
            }
        } else if (member_idx == i) && !bitflag_enabled {
            array_offset = 0;
        }

        if bit_mask == 0x80 {
            cbf_offset += 1;
            bit_mask = 1;
        } else {
            bit_mask <<= 1;
        }
    }
    array_offset as usize
}