use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::engine::{Instruction};
use crate::engine::evaluator::EvalError::NotSupport;
use crate::helpers::safe_add;

#[derive(Debug)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    NotSupport,
    InvalidPC, // 評価器の実装に誤りがある場合に発生するエラー
    // InvalidContext, // 評価器の実装に誤りがある場合に発生するエラー
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CodeGEnError: {:?}", self)
    }
}

impl Error for EvalError{}

pub fn eval(inst: &[Instruction], line: &[char], index: usize, is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, 0, index)
    } else {
        Ok(false) // 一旦対応しない
    }
}

/// 深さ優先探索で再帰的にマッチングを行う関数
fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
    index: usize,
) -> Result<bool, EvalError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvalError::InvalidPC)
        };

        match next {
            Instruction::Dollar => {
                if sp == line.len() {
                    return Ok(true)
                } else {
                    return Ok(false)
                }
            }
            Instruction::Caret => {
                if pc != 0 {
                    return Err(NotSupport)
                }
                if index != 0 {
                    return Ok(false)
                }
                safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
            }
            Instruction::Dot => {
                safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                safe_add(&mut sp, &1, || EvalError::SPOverFlow)?;
            }
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if c == sp_c {
                        // 一致した場合、次の評価
                        safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                        safe_add(&mut sp, &1, || EvalError::SPOverFlow)?;
                    } else {
                        // 一致しない場合は false
                        return Ok(false)
                    }
                } else {
                    // 最後まで来てしまったので false
                    return Ok(false);
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr
            }
            Instruction::Split(addr1, addr2) => {
                return if eval_depth(inst, line, *addr1, sp, index)? || eval_depth(inst, line, *addr2, sp, index)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}
