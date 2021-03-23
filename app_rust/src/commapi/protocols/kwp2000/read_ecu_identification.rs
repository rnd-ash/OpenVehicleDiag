use crate::commapi::protocols::{ProtocolError, ProtocolResult, ProtocolServer};

use super::{bcd_decode, bcd_decode_slice, KWP2000ECU};

pub enum IdentificationType {
    DcsEcuIdentification = 0x86,
    DcxMmcEcuIdentification = 0x87,
    OriginalVin = 0x88,
    DiagVariantCode = 0x89,
    CurrentVin = 0x90,
    CalibrationId = 0x96,
    CalibrationVerificationNumber = 0x97,
    CodeFingerprint = 0x9A,
    DataFingerprint = 0x9B,
    CodeSoftwareId = 0x9C,
    DataSoftwareId = 0x9D,
    BootSoftwareId = 0x9E,
    BootFingerprint = 0x9F,
}

#[derive(Debug, Clone)]
pub struct DcsEcuId {
    part_number: String,
    hardware_build_date: String,
    software_written_date: String,
    supplier_id: u8,
    diag_information: u16,
    production_date: String,
}

#[derive(Debug, Clone)]
pub struct DcxMmcECUId {
    pub ecu_origin: u8,
    pub supplier_id: u8,
    pub diag_information: u16,
    pub hardware_version: String,
    pub software_version: String,
    pub part_number: String,
}

#[derive(Debug, Clone)]
pub struct ToolSupplier {
    pub id: String,
    pub programming_date: String,
    pub serial_number: String,
}

#[derive(Debug, Clone)]
pub struct CodeFingerprint {
    pub num_modules: u32,
    pub active_logical_block: String,
    pub suppliers: Vec<ToolSupplier>,
}

pub fn read_dcs_id(ecu: &KWP2000ECU) -> ProtocolResult<DcsEcuId> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x86])?;
    if res.len() != 18 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 18,
            actual: res.len(),
        });
    }
    Ok(DcsEcuId {
        part_number: bcd_decode_slice(&res[2..=6]),
        // ECU hardware build date. Format WW/YY
        hardware_build_date: format!("{}/{}", bcd_decode(&res[7]), bcd_decode(&res[8])),
        /// ECU Software written date. Format WW/YY
        software_written_date: format!("{}/{}", bcd_decode(&res[9]), bcd_decode(&res[10])),
        supplier_id: res[11],
        diag_information: (res[12] as u16) << 8 | res[13] as u16,
        /// ECU production date. Format DD/MM/YY
        production_date: format!(
            "{}/{}/{}",
            bcd_decode(&res[17]),
            bcd_decode(&res[16]),
            bcd_decode(&res[15])
        ),
    })
}

pub fn read_dcx_mmc_id(ecu: &KWP2000ECU) -> ProtocolResult<DcxMmcECUId> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x87])?;
    if res.len() != 22 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 22,
            actual: res.len(),
        });
    }
    Ok(DcxMmcECUId {
        ecu_origin: res[2],
        supplier_id: res[3],
        diag_information: (res[4] as u16) << 8 | res[5] as u16,
        hardware_version: bcd_decode_slice(&res[7..=8]),
        software_version: bcd_decode_slice(&res[9..=11]),
        part_number: String::from_utf8(Vec::from(&res[12..])).unwrap_or("Unknown".into()),
    })
}

pub fn read_original_vin(ecu: &KWP2000ECU) -> ProtocolResult<String> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x88])?;
    if res.len() != 19 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 22,
            actual: res.len(),
        });
    }
    Ok(String::from_utf8(Vec::from(&res[2..])).unwrap())
}

pub fn read_variant_code(ecu: &KWP2000ECU) -> ProtocolResult<u32> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x90])?;
    if res.len() != 6 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 22,
            actual: res.len(),
        });
    }
    Ok((res[2] as u32) << 24 | (res[3] as u32) << 16 | (res[4] as u32) << 8 | res[5] as u32)
}

pub fn read_current_vin(ecu: &KWP2000ECU) -> ProtocolResult<String> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x90])?;
    if res.len() != 19 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 22,
            actual: res.len(),
        });
    }
    Ok(String::from_utf8(Vec::from(&res[2..])).unwrap())
}

pub fn read_calibration_id(ecu: &KWP2000ECU) -> ProtocolResult<String> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x96])?;
    if res.len() != 18 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 22,
            actual: res.len(),
        });
    }
    Ok(String::from_utf8(Vec::from(&res[2..])).unwrap())
}

pub fn read_calibration_verification_number(ecu: &KWP2000ECU) -> ProtocolResult<[u8; 4]> {
    let res = ecu.run_command(super::Service::ReadECUID.into(), &[0x96])?;
    if res.len() != 6 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 22,
            actual: res.len(),
        });
    }
    Ok([res[2], res[3], res[4], res[5]])
}

pub fn read_code_fingerprint(ecu: &KWP2000ECU) -> ProtocolResult<CodeFingerprint> {
    read_fingerprint(ecu, 0x9A)
}

pub fn read_data_fingerprint(ecu: &KWP2000ECU) -> ProtocolResult<CodeFingerprint> {
    read_fingerprint(ecu, 0x9B)
}

fn read_fingerprint(ecu: &KWP2000ECU, cmd: u8) -> ProtocolResult<CodeFingerprint> {
    let mut res = ecu.run_command(super::Service::ReadECUID.into(), &[cmd])?;
    if res.len() < 4 {
        return Err(ProtocolError::InvalidResponseSize {
            expect: 4,
            actual: res.len(),
        });
    }

    let num_modules = res[2];
    let active_block = res[3];

    let active_block_string = match active_block {
        0x00 => "No erase performed".into(),
        0xFE => "Erase all memory".into(),
        0xFF => "Reserved".into(),
        _ => format!("Erase block {}", active_block),
    };

    res.drain(0..3);
    let mut suppliers: Vec<ToolSupplier> = Vec::new();
    for _ in 0..num_modules {
        suppliers.push(ToolSupplier {
            id: format!("{:02X}", res[0]),
            programming_date: bcd_decode_slice(&res[1..=3]),
            serial_number: format!("{:02X}{:02X}{:02X}{:02X}", res[4], res[5], res[6], res[7]),
        });
        res.drain(0..9);
    }
    Ok(CodeFingerprint {
        num_modules: num_modules as u32,
        active_logical_block: active_block_string,
        suppliers,
    })
}
