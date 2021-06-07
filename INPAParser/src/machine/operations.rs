use std::io::Read;

use common::raf::Raf;

use super::EdiabasResult;
use super::opcode::OpCode;
use super::operand::Operand;
use super::Machine;
use super::operand::OperandDataType;

pub fn op_a2_fix(m: &mut Machine, op_code: &OpCode, arg0: &Operand, arg1: &Operand) -> EdiabasResult<()> {
    if arg0.get_data_type() != OperandDataType::Register {
        return Err(super::EdiabasError::InvalidDataType)
    }
    //let value = string_to_value(arg1.get_)
    Ok(())
}

pub fn is_printable(b: u8) -> bool {
    b.is_ascii()
}

pub fn get_string_text(data: &[u8]) -> String {
    if data.len() < 1 {
        return "{ }".into()
    }
    let mut printable = true;
    let mut length = data.len();
    for &x in data {
        if !is_printable(x) {
            printable = false;
            break;
        }
    }

    if printable && data[length-1] == 0 {
        return format!("\\{}\\", String::from_utf8_lossy(data));
    }
    return format!("{:02X?}", data)
}

pub fn get_op_arg_text(op: &Operand) -> String {
    let mut reg_name1 = String::new();
    let mut reg_name2 = String::new();
    let mut reg_name3 = String::new();
    todo!()
}

pub fn read_decrypt_bytes(raf: &mut Raf, buffer: &mut[u8], offset: usize, count: usize) {
    let raw: Vec<u8> = raf.read_bytes(count).unwrap().iter_mut().map(|x| *x ^ 0xF7).collect();
    buffer[offset..offset+count].copy_from_slice(&raw[0..count])

}

pub fn string_to_value(number: &str, valid: &mut bool) -> i64 {
    0
}