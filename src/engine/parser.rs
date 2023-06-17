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
    Char(char),
    // 1文字パターン
    Plus(Box<AST>),
    // +
    Star(Box<AST>),
    // *
    Question(Box<AST>),
    // ?
    Or(Box<AST>, Box<AST>),
    // |
    Seq(Vec<AST>), // 正規表現の列
    // abc の AST = AST::Seq(vec![AST::Char('a'), AST::Char('b'), AST::Char('c')])
}

#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char),
    // 不正なエスケープシーケンス
    InvalidRightParen(usize),
    // 開きカッコなし
    NoPrev(usize),
    // +, |, *, ? の前に式がない
    NoRightParen,
    // 閉じカッコなし
    Empty,                      // 空
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {}, char = '{}'", pos, c)
            }
            ParseError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {}", pos)
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

enum PSQ {
    Plus,
    Star,
    Question,
}

fn parse_plus_star_question(
    seq: &mut Vec<AST>,
    ast_type: PSQ,
    pos: usize,
) -> Result<(), ParseError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            PSQ::Plus => AST::Plus(Box::new(prev)),
            PSQ::Star => AST::Star(Box::new(prev)),
            PSQ::Question => AST::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParseError::NoPrev(pos)) // e.g. 先頭に + とか
    }
}

// Or で結合された複数式を AST に変換する
// e.g. abc | def | ghi => AST::Or("abc", AST::Or("def", "ghi"))
fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast))
        }
        Some(ast)
    } else {
        seq_or.pop()
    }
}

pub fn parse(expr: &str) -> Result<AST, ParseError> {
    // 内部状態を表現するための型
    // Char 状態: 文字列処理中
    // Escape 状態: エスケープシーケンス処理中
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new(); // 現在の seq コンテキスト e.g. "abc"
    let mut seq_or = Vec::new(); // 現在の Or コンテキスト(本体) e.g. "abc|de"
    let mut stack = Vec::new(); // コンテキストのスタック(一次保存)
    let mut state = ParseState::Char;  // 現在の状態

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => {
                match c {
                    '+' => parse_plus_star_question(&mut seq, PSQ::Plus, i)?,  // seq につめる, pos はエラー用 e.g AST::Plus(Box::new(seq)),
                    '*' => parse_plus_star_question(&mut seq, PSQ::Star, i)?,
                    '?' => parse_plus_star_question(&mut seq, PSQ::Question, i)?,
                    '(' => {
                        // 現在のコンテキストをスタックに保存し、
                        // 現在のコンテキストを空の状態にする
                        let prev = take(&mut seq);
                        let prev_or = take(&mut seq_or);
                        stack.push((prev, prev_or));
                    }
                    ')' => {
                        // 現在のコンテキストをスタックからポップ
                        if let Some((mut prev, prev_or)) = stack.pop() {
                            // "()" のように、式が殻の場合は push しない "(abc|de|)"とかもかな..なんでエラーちゃうんやろ？再利用用？
                            if !seq.is_empty() {
                                seq_or.push(AST::Seq(seq))
                            }

                            // Or を生成 e.g. AST::Or("abc", AST::Or("def", "ghi"))
                            if let Some(ast) = fold_or(seq_or) {
                                prev.push(ast);
                            }

                            // 以前のコンテキストを 現在のコンテキストにする
                            seq = prev;
                            seq_or = prev_or; // ??これが残っていることある？ abc|(ed)とかはそうなりそう
                        } else {
                            // "abc)" のように開きカッコがない場合はエラー
                            return Err(ParseError::InvalidRightParen(i)); // MEMO: Boxはいりません
                        }
                    }
                    '|' => {
                        if seq.is_empty() {
                            // "||" や "(|abc)" など式が空の場合はエラー
                            return Err(ParseError::NoPrev(i));
                        } else {
                            // 現在のコンテキストを空の状態にして、
                            // Or コンテキスト に入れる
                            let prev = take(&mut seq);
                            seq_or.push(AST::Seq(prev));
                        }
                    }
                    '\\' => state = ParseState::Escape,
                    _ => seq.push(AST::Char(c)),
                }
            }
            ParseState::Escape => {
                // エスケープシーケンス処理
                let ast = parse_escape(i, c)?;
                seq.push(ast);
                state = ParseState::Char;
            }
        }
    }

    // 最終処理

    // 閉じカッコが足りない場合はエラー
    if !stack.is_empty() {
        return Err(ParseError::NoRightParen);
    }

    // "()" のように、式が空の場合は push しない
    // 最後の文字列はここでpushされる
    if !seq.is_empty() {
        seq_or.push(AST::Seq(seq));
    }

    // Or を生成し、成功した場合はそれを返す
    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(ParseError::Empty)
    }
}