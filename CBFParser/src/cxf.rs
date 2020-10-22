#[allow(dead_code)]
use std::io::BufReader;
use std::io::Read;
use common::raf::*;

const CLT1: [u8; 13] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT2: [u8; 27] = [4, 4, 4, 2, 2, 4, 4, 4, 4, 4, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 2, 1];
const CLT3: [u8; 33] = [6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2];
const CLT4: [u8; 18] = [6, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 4, 4, 4, 4];
const CLT5: [u8; 14] = [4, 4, 2, 2, 4, 4, 2, 1, 4, 4, 4, 4, 4, 4];
const CLT6: [u8; 26] = [4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 1, 1, 1, 1, 1, 5, 1, 1, 1, 1, 4, 4, 4, 4, 4];
const CLT7: [u8; 39] = [6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1, 1, 1, 1, 4, 4, 4, 2, 4, 4, 4];
const CLT8: [u8; 13] = [2, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT9: [u8; 9] = [2, 4, 4, 4, 4, 2, 4, 2, 4];
const CLT10: [u8; 12] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2];
const CLT11: [u8; 9] = [2, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT12: [u8; 4] = [2, 4, 4, 4];
const CLT13: [u8; 12] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT14: [u8; 6] = [2, 4, 4, 4, 1, 1];
const CLT15: [u8; 11] = [2, 4, 2, 2, 2, 4, 2, 2, 4, 4, 4];
const CLT16: [u8; 11] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 1, 1];
const CLT17: [u8; 22] = [6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 1];
const CLT18: [u8; 13] = [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT19: [u8; 14] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT20: [u8; 14] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT21: [u8; 5] = [2, 4, 4, 4, 4];
const CLT22: [u8; 11] = [2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2];
const CLT23: [u8; 6] = [2, 4, 4, 4, 4, 4];
const CLT24: [u8; 26] = [6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT25: [u8; 5] = [2, 4, 4, 4, 4];
const CLT26: [u8; 7] = [2, 4, 4, 4, 4, 4, 4];
const CLT27: [u8; 3] = [2, 4, 4];
const CLT28: [u8; 13] = [2, 2, 4, 4, 2, 4, 4, 2, 4, 4, 2, 4, 4];
const CLT29: [u8; 8] = [2, 4, 4, 4, 4, 4, 4, 4];
const CLT30: [u8; 3] = [2, 4, 4];
const CLT31: [u8; 9] = [2, 4, 4, 2, 4, 4, 4, 4, 4];
const CLT32: [u8; 6] = [2, 4, 2, 4, 4, 4];
const CLT33: [u8; 19] = [6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2];
const CLT34: [u8; 13] = [6, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4];
const CLT35: [u8; 8] = [2, 4, 4, 4, 4, 4, 2, 4];
const CLT36: [u8; 8] = [2, 4, 4, 4, 4, 4, 2, 4];
const CLT37: [u8; 9] = [2, 4, 4, 4, 4, 4, 4, 2, 4];
const CLT38: [u8; 13] = [2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 0, 0, 0];

const CBF_LOOKUP_TABLE: [&[u8]; 38] = [
    &CLT1, &CLT2, &CLT3, &CLT4, &CLT5, &CLT6, &CLT7, &CLT8, &CLT9, &CLT10,
    &CLT11, &CLT12, &CLT13, &CLT14, &CLT15, &CLT16, &CLT17, &CLT18, &CLT19, &CLT20,
    &CLT21, &CLT22, &CLT23, &CLT24, &CLT25, &CLT26, &CLT27, &CLT28, &CLT29, &CLT30,
    &CLT30, &CLT32, &CLT33, &CLT34, &CLT35, &CLT36, &CLT37, &CLT38
];

#[derive(Debug)]
pub enum CxfHeader {
    CBF {
        translator_version: String,
        date: String,
        fingerprint: String,
        name: String,
        language: String,
        target_release: String,
        gpd_translator_version: String,
    },
    CFF {
        translator_version: String,
        date: String,
        fingerprint: String,
        name: String,
        language: String
    }
}

#[derive(Debug)]
pub enum CxfType {
    CFF,
    CBF
}

pub struct CxfFile {
    raf: Raf,
    pub f_type: CxfType
}

pub struct CffFile {
    base: CxfFile
}

pub struct CbfFile {
    base: CxfFile
}

pub trait Cxf {
    fn from_cxf(cxf: CxfFile) -> Option<Box<Self>>;
    fn read_header(&mut self) -> Result<CxfHeader>;
}

impl CxfFile {
    pub fn from_file<T: Read>(f: &mut BufReader<T>) -> std::io::Result<Self> {
        Raf::from_read(f, RafByteOrder::LE).map(|mut r| {
            // Read CxF type
            let c_type = match r.seek_read(0x01, Raf::read_byte).unwrap() {
                0x42 => CxfType::CBF,
                0x46 => CxfType::CFF,
                _ => panic!("File is not known CxF type")
            };
            println!("{:?}", c_type);
            r.seek(0x00).unwrap(); // Back to start
            CxfFile {
                raf: r,
                f_type: c_type
            }
        })
    }

    pub fn read_meta_data(&mut self) -> Result<Vec<String>> {
        self.raf.seek(0)?;
        // Read till end of metadata block
        let mut bytes = self.raf.read_until(0x00)?;
        bytes.pop();
        match String::from_utf8(bytes) {
            Ok(s) => Ok(s.lines().map(String::from).collect()),
            Err(_) => Err(RafError::StrParseError)
        }
    }

    pub fn get_offset(m_type: u16, s_type: u16, cxf_bytes: u16) -> u16 {
        let struct_memeber_offsets = CBF_LOOKUP_TABLE[s_type as usize];
        let mut cxf_offset: u16 = 0;
        let mut bitmask: u16 = 1;
        // Start at element 1
        let mut array_ptr: u16 = struct_memeber_offsets[0] as u16;
        (1..=m_type).for_each(|i| {
            // Only want the lower byte
            // Read byte from CBF file, AND it with bitmask to see if it matches the requested type
            if (bitmask & 0x00FF) & ((cxf_bytes + cxf_offset) & 0x00FF) != 0 {
                if m_type != i {
                    // Not this element, must be next, add bytes to skip
                    array_ptr += struct_memeber_offsets[i as usize] as u16;
                }
            } else if m_type == i && (bitmask & 0x00FF) & ((cxf_bytes + cxf_offset) & 0x00FF) == 0 {
                // reach limit, no results found
                array_ptr = 0;
            }
            // Bit mask has rolled over! (0b1000 0000), move it back to start (0b0000 0001)
            if bitmask == 0x80 {
                cxf_offset += 1;
                bitmask = 1;
            } else {
                // Shift bitmask by 1 bit
                bitmask = bitmask << 1;
            }
        });
        array_ptr
    }
}

impl Cxf for CffFile {
    fn from_cxf(cxf: CxfFile) -> Option<Box<Self>> {
        match cxf.f_type {
            CxfType::CBF => None,
            CxfType::CFF => {
                Some(Box::new(
                    CffFile { base: cxf }
                ))
            }
        }
    }

    fn read_header(&mut self) -> Result<CxfHeader> {
        let mut lines = self.base.read_meta_data()?.into_iter();
        Ok(CxfHeader::CFF{
            translator_version: lines.find(|s| s.contains("CFF-TRANSLATOR-VERSION:")).unwrap(),
            date: lines.find(|s| s.contains("DATE:")).unwrap(),
            fingerprint: lines.find(|s| s.contains("FINGERPRINT:")).unwrap(),
            name: lines.find(|s| s.contains("CFF:")).unwrap(),
            language: lines.find(|s| s.contains("LANGUAGE:")).unwrap(),
        })
    }
}

impl CffFile {

}

impl Cxf for CbfFile {
    fn from_cxf(cxf: CxfFile) -> Option<Box<Self>> {
        match cxf.f_type {
            CxfType::CFF => None,
            CxfType::CBF => {
                Some(Box::new(
                    CbfFile { base: cxf }
                ))
            }
        }
    }
    fn read_header(&mut self) -> Result<CxfHeader> {
        let mut lines = self.base.read_meta_data()?.into_iter();
        Ok(CxfHeader::CBF{
            translator_version: lines.find(|s| s.contains("CFF-TRANSLATOR-VERSION:")).unwrap(),
            date: lines.find(|s| s.contains("DATE:")).unwrap(),
            fingerprint: lines.find(|s| s.contains("FINGERPRINT:")).unwrap(),
            name: lines.find(|s| s.contains("CFF:")).unwrap(),
            language: lines.find(|s| s.contains("LANGUAGE:")).unwrap(),
            target_release: lines.find(|s| s.contains("TARGET-RELEASE:")).unwrap(),
            gpd_translator_version: lines.find(|s| s.contains("GPD-TRANSLATOR-VERSION:")).unwrap(),
        })
    }
}

impl CbfFile {

}