use crate::machine::operand::OperandDataType;

use super::{
    opcode::OpCode,
    operand::{OpAddrMode, Operand, OperandData},
    EdiabasError, EdiabasResult, Machine,
};

pub fn op_move(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    let data_type_0 = arg0.get_data_type();
    let data_type_1 = arg1.get_data_type();

    todo!();
}

pub fn op_clear(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_comp(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_subb(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_adds(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_mult(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_divs(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_and(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_or(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xor(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_not(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jump(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jae(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jz(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jnz(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jv(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jnv(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jmi(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jpl(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_clrc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_setc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_asr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_lsl(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_lsr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_asl(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_nop(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_eoj(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_push(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    let mut value = arg0.get_value_data(0, m)?;
    let length = arg0.get_data_length(m, false)?;
    for _ in 0..length {
        m.stack.push_back((value & 0xFF) as u8);
        value = value >> 8;
    }
    Ok(())
}

pub fn op_pop(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    if arg0.data1.get_data_type() != OperandDataType::Register {
        return Err(EdiabasError::InvalidDataType("op_funcs", "op_pop"));
    }
    if arg0.get_data_type() != OperandDataType::Integer {
        return Err(EdiabasError::InvalidDataType("op_funcs", "op_pop"));
    }

    let mut value: u32 = 0;
    let length = arg0.get_data_length(m, false)?;
    if m.stack.len() < length as usize {
        // Set error EDIABAS_BIP_0005
        return Ok(());
    }
    for _ in 0..length {
        value <<= 8;
        value |= m.stack.pop_back().unwrap() as u32;
    }
    arg0.set_raw_data(m, OperandData::Integer(value), 1);
    m.flags.set_overflow_bit(false);
    m.flags.update_flags(value, length);
    Ok(())
}

pub fn op_scmp(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_scat(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_scut(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_slen(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_spaste(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_serase(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    if let OperandData::Register(r) = &arg0.data1 {
        let mut start_idx: u32;
        match arg0.addr_mode {
            OpAddrMode::IdxImm => {
                start_idx = arg0.get_int_data(m)?;
            }
            OpAddrMode::IdxReg => {
                todo!()
                //Ok(())
            }
            _ => return Err(EdiabasError::InvalidDataType("op_funcs", "op_serase")),
        }
        todo!();
        Ok(())
    } else {
        Err(EdiabasError::InvalidDataType("op_funcs", "op_serase"))
    }
}

pub fn op_xconnect(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    println!("Simulating interface connect");
    Ok(())
}

pub fn op_xhangup(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xsetpar(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xawlen(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xsend(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xsendf(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xrequf(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xstopf(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    println!("Stopping frequent");
    Ok(())
}

pub fn op_xkeyb(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xstate(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xreset(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xtype(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xvers(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergb(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergw(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergd(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergi(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergs(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_a2_flt(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fadd(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fsub(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fmul(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fdiv(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergy(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_enewset(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_etag(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xreps(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_gettmr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_settmr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_sett(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_clrt(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jt(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jnt(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_addc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_subc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_break(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_clrv(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_eerr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_popf(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_pushf(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_atsp(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_swap(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_setspc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_srevrs(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_stoken(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_parl(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_pars(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_close(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jg(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jge(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jl(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jle(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ja(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_jbe(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_open(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_read(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_readln(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_seek(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_seekln(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_tell(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_f_tellln(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_a2_fix(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fix_2_flt(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_parr(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_test(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_wait(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_date(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_time(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xbatt(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn n_noneull(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xgetport(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xignit(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_xraw(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fix_2_hex(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_fix_2_dez(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_tabset(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_tabseek(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_tabget(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_strcat(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_pary(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_parn(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergc(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_ergl(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}

pub fn op_tabline(
    m: &mut Machine,
    oc: &mut OpCode,
    arg0: &mut Operand,
    arg1: &mut Operand,
) -> EdiabasResult<()> {
    todo!();
}
