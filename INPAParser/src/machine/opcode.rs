
pub struct OpCode<'a> {
    op_code: u8,
    pneumonic: &'a str,
    op_func: &'a super::OperationDelegate,
    arg0_is_near_addr: bool
}

impl<'a> OpCode<'a> {
    pub fn new(op_code: u8, pneumonic: &'a str, op_func: &'a super::OperationDelegate, arg0_is_near_addr: bool) -> Self {
        Self {
            op_code,
            pneumonic,
            op_func,
            arg0_is_near_addr
        }
    }
}