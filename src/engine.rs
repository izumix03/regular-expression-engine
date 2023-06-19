use std::fmt::{Display, Formatter};
use crate::helpers::DynError;

mod parser;
mod codegen;
mod evaluator;

#[derive(Debug)]
pub enum Instruction {
    Char(char),
    Match,
    Jump(usize),
    Split(usize, usize), // L1のアドレス、L2のアドレス
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Char(c) => write!(f, "char {}", c),
            Instruction::Match => write!(f, "match"),
            Instruction::Jump(addr) => write!(f, "jump {:>04}", addr),
            Instruction::Split(addr1, addr2) => write!(f, "split {:>04}, {:>04}", addr1, addr2),
        }
    }
}

/// 正規表現と文字列をマッチング
/// # 利用例
/// ```
/// use regex;
/// regex::do_matching("abc | (de|cd)+", "decdddede", true);
///
/// # 引数
/// expr 正規表現
/// line マッチ対象の文字列
/// is_depth 深さ優先探索かどうか
///
/// # 返り値
/// エラーなく実行してマッチング成功したら true
/// エラーなく実行してマッチング失敗したら false
/// エラーがある場合は Err
pub fn do_matching(expr: &str, line: &str, is_depth: bool) -> Result<bool,DynError>{
    let ast = parser::parse(expr)?;
    let code = codegen::get_code(&ast)?;
    let line = line.chars().collect::<Vec<char>>();
    Ok(evaluator::eval(&code, &&line, is_depth)?)
}