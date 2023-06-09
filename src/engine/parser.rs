//! 正規表現の式をパースし、抽象構文木に変換
//  ↑ cargo doc でドキュメント化される
use std::{
    error::Error, // エラー用の型を規定するためのトレイト
    fmt::{self, Display}, // println! マクロなどで表示するためのトレイト
    mem::take, // ある変数から所有権の取得し、その変数の初期化を同時に行う関数
};
use std::fmt::{Formatter, write};

#[derive(Debug)]
pub enum AST {
    Char(char), // 1文字パターン
    Plus(Box<AST>), // +
    Star(Box<AST>), // *
    Question(Box<AST>), // ?
    Or(Box<AST>, Box<AST>), // |
    Seq(Vex<AST>), // 正規表現の列
    // abc の AST = AST::Seq(vec![AST::Char('a'), AST::Char('b'), AST::Char('c')])
}

#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char), // 不正なエスケープシーケンス
    InvalidRightParen(usize),   // 開きカッコなし
    NoPrev(usize),              // +, |, *, ? の前に式がない
    NoRightParen,               // 閉じカッコなし
    Empty,                      // 空
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParseError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}")
            }
            ParseError::NoRightParen => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty => write!(f, "ParseError: empty expression"),
        }
    }
}

impl Error for ParseError {}

// pos: 現在の文字の位置
// c: エスケープする特殊文字
fn parse_escape(pos: usize, c: char) -> Result<AST, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}