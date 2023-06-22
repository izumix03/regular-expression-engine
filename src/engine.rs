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
/// regex::do_matching("abc|(de|cd)+", "decddede", true);
/// ```
///
/// # 引数
/// expr → 正規表現
/// line → マッチ対象の文字列
/// is_depth → 深さ優先探索かどうか
///
/// # 返り値
/// エラーなく実行してマッチング成功したら true
/// エラーなく実行してマッチング失敗したら false
/// エラーがある場合は Err
pub fn do_matching(expr: &str, line: &str, is_depth: bool) -> Result<bool,DynError>{
    let ast = parser::parse(expr)?; // AST変換
    let code = codegen::get_code(&ast)?; // 命令に変換
    let line = line.chars().collect::<Vec<char>>();
    Ok(evaluator::eval(&code, &&line, is_depth)?) // 正規表現評価
}

/// 正規表現をパースしてコード生成し、
/// ASTと命令列を標準出力に表示。
///
/// # 利用例
///
/// ```
/// use regex;
/// regex::print("abc|(de|cd)+");
/// ```
///
/// # 返り値
///
/// 入力された正規表現にエラーがあったり、内部的な実装エラーがある場合はErrを返す。
pub fn print(expr: &str) -> Result<(), DynError> {
    println!("expr: {expr}");
    let ast = parser::parse(expr)?;
    println!("AST: {:?}", ast);

    println!();
    println!("code:");
    let code = codegen::get_code(&ast)?;
    for (n, c) in code.iter().enumerate() {
        println!("{:>04}: {c}", n);
    }

    Ok(())
}