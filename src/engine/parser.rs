//! parser.rs parses a string (regex expression) into AST (Abstract Syntax Tree).

use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};

/// パースエラーを表現するための型
#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char), // 誤ったエスケープシーケンス
    InvalidRightParen(usize),   // 開き括弧なし
    NoPrev(usize),              // +, |, *, ? の前に式がない
    NoRightParen,               // 閉じ括弧がない
    Empty,                      // 空のパターン
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// 抽象構文木を表現するための型
#[derive(Debug)]
pub enum AST {
    Char(char),             // 単一の文字
    Plus(Box<AST>),         // +: 1回以上の繰り返し
    Star(Box<AST>),         // *: 0回以上の繰り返し
    Question(Box<AST>),     // ?: 0回または1回の繰り返し
    Or(Box<AST>, Box<AST>), // |: 選択肢
    Seq(Vec<AST>),          // 正規表現のまとまり
}

/// parse_plus_star_question関数で利用する
enum PSQ {
    Plus,
    Star,
    Question,
}

/// 正規表現の文字列をパースしてASTを生成する関数
pub fn parse(expr: &str) -> Result<AST, ParseError> {
    // 内部状態を表現するための型
    // Char: 文字列処理中
    // Escape: エスケープシーケンス処理中
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new(); // 現在のSeqのコンテキスト
    let mut seq_or = Vec::new(); // 現在のOrのコンテキスト
    let mut stack = Vec::new(); // コンテキストのスタック
    let mut state = ParseState::Char; // 初期状態は文字列処理中

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => {
                match c {
                    '+' => parse_plus_star_question(&mut seq, PSQ::Plus, i)?,
                    '*' => parse_plus_star_question(&mut seq, PSQ::Star, i)?,
                    '?' => parse_plus_star_question(&mut seq, PSQ::Question, i)?,
                    '(' => {
                        // 現在のコンテキストをスタックに保存
                        // 現在のコンテキストを空の状態にする
                        let prev = take(&mut seq);
                        let perv_or = take(&mut seq_or);
                        stack.push((prev, perv_or));
                    }
                    ')' => {
                        // 現在のコンテキストをスタックからポップ
                        if let Some((mut prev, prev_or)) = stack.pop() {
                            // "()" のように，式が空の場合はpushしない
                            if !seq.is_empty() {
                                seq_or.push(AST::Seq(seq));
                            }
                            // orを生成
                            if let Some(ast) = fold_or(seq_or) {
                                prev.push(ast);
                            }

                            // 以前のコンテキストを現在のコンテキストにする
                            seq = prev;
                            seq_or = prev_or;
                        } else {
                            return Err(ParseError::InvalidRightParen(i));
                        }
                    }
                    '|' => {
                        if seq.is_empty() {
                            return Err(ParseError::NoPrev(i));
                        } else {
                            let prev = take(&mut seq);
                            seq_or.push(AST::Seq(prev));
                        }
                    }
                    '\\' => state = ParseState::Escape, // エスケープシーケンスの開始
                    _ => seq.push(AST::Char(c)),
                };
            }
            ParseState::Escape => {
                let ast = parse_escape(i, c)?;
                seq.push(ast);
                state = ParseState::Char; // エスケープ処理が終わったので、状態を戻す
            }
        }
    }

    if !stack.is_empty() {
        return Err(ParseError::NoRightParen);
    }

    // "()" のように，式がからの場合はpushしない
    if !seq.is_empty() {
        seq_or.push(AST::Seq(seq));
    }

    // Orを生成し，成功した場合はそれを返す
    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(ParseError::Empty)
    }
}

/// +, *, ? をASTに変換する
///
/// 後置記法で，+, *, ?の前にパターンがない場合はエラー
/// 例: *ab, abc|+ などはエラー
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
        Err(ParseError::NoPrev(pos))
    }
}

/// 特殊文字のエスケープを処理する関数
fn parse_escape(pos: usize, c: char) -> Result<AST, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}

/// Orで結合された複数の式をASTに変換
///
/// たとえば，abc|def|ghi はAST::Or("abc", AST::Or("def", "ghi"))というASTとなる
fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        // seq_orの要素が2つ以上ある場合はOrで結合
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse(); // AST::Orの先頭をASTのルートにするために，反転する
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        // seq_orの要素が1つのみの場合は，Orではなく，最初の値を返す
        seq_or.pop()
    }
}
