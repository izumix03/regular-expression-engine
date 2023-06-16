use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::engine::Instruction;
use crate::engine::parser::AST;
use crate::helpers::safe_add;

// コード生成エラーを表す
#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError{}


// コード生成器
#[derive(Default, Debug)]
struct Generator {
    pc: usize, // プログラムカウンタ
    insts: Vec<Instruction>
}


impl Generator {
    // プログラムカウンタをインクリメント
    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
    }

    // ASTをパターン分けして、コード生成を行う
    fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::Or(e1, e2) => self.gen_or(e1, e2)?,
            AST::Plus(e) => self.gen_plus(e)?,
            AST::Star(e) => self.gen_star(e)?,
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,
        }

        Ok(())
    }

    fn gen_seq(&mut self, exprs: &[AST]) -> Result<(), CodeGenError> {
        for e in exprs {
            self.gen_expr(e)?
        }
        Ok(())
    }

    fn gen_char(&mut self, c: char) -> Result<(), CodeGenError> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_or(&mut self, e1: &AST, e2: &AST) -> Result<(), CodeGenError> {
        // L1とL2に分ける
        let split_addr = self.pc;

        self.inc_pc()?; // =====================
        let split = Instruction::Split(self.pc, 0); // L1 = pc, L2 = 仮に0
        self.insts.push(split);

        // L1: e1のコード
        self.gen_expr(e1);

        // jmp L3
        let jmp_addr = self.pc;
        self.insts.push(Instruction::Jump(0)); // L3 を仮に0と設定

        // L2の値を設定
        self.inc_pc()?; // =====================
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            // 仮に0とおいていたのでインクリメントして設定
            *l2 = self.pc
        } else {
            return Err(CodeGenError::FailOr)
        }

        // L2: e2のコード
        self.gen_expr(e2)?;

        // L3の値を設定
        if let Some(Instruction::Jump(l3)) = self.insts.get_mut(jmp_addr) {
            *l3 = self.pc;
        } else {
            return Err(CodeGenError::FailOr)
        }

        Ok(())
    }
}