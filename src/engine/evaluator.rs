use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::engine::Instruction;
use crate::helpers::safe_add;

#[derive(Debug)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    InvalidPC, // 評価器の実装に誤りがある場合に発生するエラー
    InvalidContext, // 評価器の実装に誤りがある場合に発生するエラー
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CodeGEnError: {:?}", self)
    }
}

impl Error for EvalError{}

pub fn eval(inst: &[Instruction], line: &[char], is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, 0)
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
) -> Result<bool, EvalError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvalError::InvalidPC)
        };

        match next {
            Instruction::Char(c) => {
                if let some(sp_c) = line.get(sp) {
                    if c == ap_c {
                        // 一致した場合、次の評価
                        safe_add(&mut pc, &1, || Box::new(EvalError::PCOverFlow))?;
                        safe_add(&mut sp, &1, || Box::new(EvalError::SPOverFlow))?;
                    } else {
                        // 一致しない場合は false で終わり
                        return Ok(false)
                    }
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr
            }
            Instruction::Split(addr1, addr2) => {
                return if eval_depth(isnt, line, *addr1, sp)? ||
                    eval_depth(inst, line, *addr2, sp)? {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}
