use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::engine::Instruction;
use crate::engine::parser::AST;
use crate::helpers::safe_add;

pub fn get_code(ast: &AST) -> Result<Vec<Instruction>, CodeGenError> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}

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
            AST::Dot => self.gen_dot()?,
            AST::Or(e1, e2) => self.gen_or(e1, e2)?,
            AST::Plus(e) => self.gen_plus(e)?,
            AST::Star(e) => self.gen_star(e)?,
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,
            AST::Caret => self.gen_caret()?,
            AST::Dollar => self.gen_dollar()?,
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

    fn gen_dot(&mut self)  -> Result<(), CodeGenError> {
        let inst = Instruction::Dot;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_caret(&mut self) -> Result<(), CodeGenError> {
        let inst = Instruction::Caret;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_dollar(&mut self) -> Result<(), CodeGenError> {
        let inst = Instruction::Dollar;
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
        self.gen_expr(e1)?;

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

    /// ?限量子のコード生成器。
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    ///     split L1, L2
    /// L1: eのコード
    /// L2:
    /// ```
    fn gen_question(&mut self, e: &AST) -> Result<(), CodeGenError> {
        // split L1, L2
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0); // self.pcがL1。L2を仮に0と設定
        self.insts.push(split);

        // L1: eのコード
        self.gen_expr(e)?;

        // L2の値を設定
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
            Ok(())
        } else {
            Err(CodeGenError::FailQuestion)
        }
    }

    /// 以下のようなコードを生成
    ///
    /// ```text
    /// L1: eのコード
    ///     split L1, L2
    /// L2:
    /// ```
    fn gen_plus(&mut self, e: &AST) -> Result<(), CodeGenError> {
        // L1: eのコード
        let l1 = self.pc;
        self.gen_expr(e)?;

        // split L1, L2
        self.inc_pc()?;
        let split = Instruction::Split(l1, self.pc); // self.pcがL2
        self.insts.push(split);

        Ok(())
    }

    /// *限量子のコード生成器。
    ///
    /// 以下のようなコードを生成
    ///
    /// ```text
    /// L1: split L2, L3
    /// L2: eのコード
    ///     jump L1
    /// L3:
    /// ```
    fn gen_star(&mut self, e: &AST) -> Result<(), CodeGenError> {
        // L1: split L2, L3
        let l1 = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0); // self.pcがL2。L3を仮に0と設定
        self.insts.push(split);

        // L2: eのコード
        self.gen_expr(e)?;

        // jump L1
        self.inc_pc()?;
        self.insts.push(Instruction::Jump(l1));

        // L3の値を設定
        if let Some(Instruction::Split(_, l3)) = self.insts.get_mut(l1) {
            *l3 = self.pc;
            Ok(())
        } else {
            Err(CodeGenError::FailStar)
        }
    }

    // コード生成を行う関数の入り口
    fn gen_code(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        self.gen_expr(ast)?;
        // 最後に Match を作成する
        self.inc_pc()?;
        self.insts.push(Instruction::Match);
        Ok(())
    }
}

