use super::Instruction;
use crate::helper::safe_add;
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self, Display},
};

#[derive(Debug)]
pub enum EvalError {
    PCOverflow,
    SPOverflow,
    InvalidPC,
    InvalidContext,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EvalError: {:?}", self)
    }
}

impl Error for EvalError {}

fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvalError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvalError::InvalidPC);
        };

        match next {
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if c == sp_c {
                        safe_add(&mut pc, &1, || EvalError::PCOverflow)?;
                        safe_add(&mut sp, &1, || EvalError::SPOverflow)?;
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                if eval_depth(inst, line, *addr1, sp)? || eval_depth(inst, line, *addr2, sp)? {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
        }
    }
}

pub fn eval(inst: &[Instruction], line: &[char], is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, 0)
    } else {
        eval_width(inst, line)
    }
}

fn pop_ctx(
    pc: &mut usize,
    sp: &mut usize,
    ctx: &mut VecDeque<(usize, usize)>,
) -> Result<(), EvalError> {
    if let Some((new_pc, new_sp)) = ctx.pop_back() {
        *pc = new_pc;
        *sp = new_sp;
        Ok(())
    } else {
        Err(EvalError::InvalidContext)
    }
}

fn eval_width(inst: &[Instruction], line: &[char]) -> Result<bool, EvalError> {
    let mut ctx = VecDeque::new();
    let mut pc = 0;
    let mut sp = 0;

    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvalError::InvalidPC);
        };

        match next {
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if c == sp_c {
                        safe_add(&mut pc, &1, || EvalError::PCOverflow)?;
                        safe_add(&mut sp, &1, || EvalError::SPOverflow)?;
                    } else {
                        if ctx.is_empty() {
                            return Ok(false);
                        } else {
                            pop_ctx(&mut pc, &mut sp, &mut ctx)?;
                        }
                    }
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                pc = *addr1;
                ctx.push_back((*addr2, sp));
                continue;
            }
        }

        if !ctx.is_empty() {
            ctx.push_back((pc, sp));
            pop_ctx(&mut pc, &mut sp, &mut ctx)?;
        }
    }
}
