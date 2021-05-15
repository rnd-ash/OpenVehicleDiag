use crate::commapi::protocols::{ProtocolResult, ProtocolServer};

use super::UDSECU;

pub fn read_variant_id(ecu: &UDSECU) -> ProtocolResult<u32> {
    let mut res = ecu.run_command(super::UDSCommand::ReadDataByID.into(), &[0xF1, 0x00])?;
    println!("{:02X?}", res);
    res.drain(0..2);
    Ok((res[0] as u32) << 24 | (res[1] as u32) << 16 | (res[2] as u32) << 8 | res[3] as u32)
}
