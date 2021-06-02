use self::{opcode::OpCode, operand::Operand, string_data::StringData};

pub (crate) mod register;
pub (crate) mod flag;
pub (crate) mod operations;
pub (crate) mod operand;
pub (crate) mod arg_info;
pub (crate) mod opcode;
pub (crate) mod string_data;


pub type OperationDelegate = fn(m: &Machine, oc: &OpCode, arg0: &Operand, arg1: &Operand) -> ();
pub type VJobDelegate = fn(m: &Machine, ) -> ();
pub type AbortJobDelegate = fn() -> bool;
pub type ProgressJobDelegate = fn(m: &Machine) -> ();
pub type ErrorRaisedDelegate = fn() -> ();

pub enum EdiabasError {
    InvalidDataLength,
    InvalidDataType,
    NullData,
}

pub type Result<T> = std::result::Result<T, EdiabasError>;

#[derive(Debug, Clone)]
pub struct Machine<'a> {
    pub float_registers: Vec<f32>,
    pub string_registers: Vec<StringData<'a>>,
    pub byte_registers: Vec<u8>
}