//! parser.rs parses a string (regex expression) into AST (Abstract Syntax Tree).
use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};

/// 抽象構文木を表現するための型
#[derive(Debug)]
pub enum AST {
    Char(char),         // 単一の文字
    Plus(Box<AST>),     // +: 1回以上の繰り返し
    Star(Box<AST>),     // *: 0回以上の繰り返し
    Question(Box<AST>), // ?: 0回または1回の繰り返し
    Or(Box<AST>),       // |: 選択肢
    Seq(Vec<AST>),      // 正規表現のまとまり
}

/// パースエラーを表現するための型
#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char), // 誤ったエスケープシーケンス
    InvalidRightParem(usize),   // 開き括弧なし
    NoPrev(usize),              // +, |, *, ? の前に式がない
    NoRightParem,               // 閉じ括弧がない
    Empty,                      // 空のパターン
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParseError::InvalidRightParem(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}")
            }
            ParseError::NoRightParem => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty => write!(f, "ParseError: empty expression"),
    }
}

impl Error for ParseError {}
