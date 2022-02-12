use logos::{Lexer, Logos};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum LexicalError {
    #[error("invalid int literal: {0}")]
    InvalidIntLiteral(#[from] std::num::ParseIntError),
    #[error("invalid float literal: {0}")]
    InvalidFloatLiteral(#[from] std::num::ParseFloatError),
}


fn parse_str<'a>(lex: &mut logos::Lexer<'a, Token<'a>>) -> &'a str {
    let s = lex.slice();
    &s[1..s.len() - 1]
}

fn parse_num<'a>(lex: &mut logos::Lexer<'a, Token<'a>>, unit: Option<Unit>) -> Number {
    let s: &str = lex.slice();
    let value = s.parse::<f64>().unwrap();
    Number {
        value,
        unit
    }
}

#[derive(Logos, Debug, PartialEq)]
pub enum Token<'input> {
    #[regex(r#"[a-zA-Z_][a-zA-Z0-9_]*"#)]
    Ident,

    //------------------- Keywords -------------------
    #[token("color")]
    Color,
    #[token("paint")]
    Paint,
    #[token("style")]
    Style,
    #[token("border")]
    Border,


    //------------------- Literals -------------------
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    BoolLit(bool),

    #[regex(r#""([^\\"]*)""#, parse_str)]
    Str(&'input str),

    #[regex("[+-]?[0-9]*[.]?[0-9]+(?:[eE][+-]?[0-9]+)?", |tk| parse_num(tk, None))]
    #[regex("[+-]?[0-9]*[.]?[0-9]+(?:[eE][+-]?[0-9]+)?dip", |tk| parse_num(tk, Some(Unit::Dip)))]
    #[regex("[+-]?[0-9]*[.]?[0-9]+(?:[eE][+-]?[0-9]+)?em", |tk| parse_num(tk, Some(Unit::Em)))]
    #[regex("[+-]?[0-9]*[.]?[0-9]+(?:[eE][+-]?[0-9]+)?px", |tk| parse_num(tk, Some(Unit::Px)))]
    #[regex("[+-]?[0-9]*[.]?[0-9]+(?:[eE][+-]?[0-9]+)?in", |tk| parse_num(tk, Some(Unit::In)))]
    #[regex("[+-]?[0-9]*[.]?[0-9]+(?:[eE][+-]?[0-9]+)?%", |tk| parse_num(tk, Some(Unit::Percentage)))]
    Number(Number),

    #[regex("^#(?:[0-9a-f]{3}){1,2}$|^#(?:[0-9a-f]{4}){1,2}$")]
    HexColor(&'input str),

    //------------------- Comments -------------------
    #[regex("//.*", logos::skip)]
    SingleLineComment,
    #[regex(r"/\*([^*]|\*[^/])+\*/", logos::skip)]
    BlockComment,

    //------------------- Operators -------------------
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(".")]
    Dot,
    #[token("++")]
    Inc,
    #[token("--")]
    Dec,
    #[token("+")]
    Plus,
    #[token("-")]
    Dash,
    #[token("!")]
    Bang,
    #[token("~")]
    Tilde,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("<<")]
    LShift,
    #[token(">>")]
    RShift,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token("<=")]
    LEqual,
    #[token(">=")]
    REqual,
    #[token("==")]
    DEqual,
    #[token("!=")]
    BangEqual,
    #[token("&")]
    Ampersand,
    #[token("^")]
    Caret,
    #[token("|")]
    Bar,
    #[token("&&")]
    And,
    #[token("^^")]
    Xor,
    #[token("||")]
    Or,
    #[token("?")]
    Question,
    #[token(":")]
    Colon,
    #[token("=")]
    Equal,
    #[token("*=")]
    StarEqual,
    #[token("/=")]
    SlashEqual,
    #[token("%=")]
    PercentEqual,
    #[token("+=")]
    PlusEqual,
    #[token("-=")]
    DashEqual,
    #[token("<<=")]
    LShiftEqual,
    #[token(">>=")]
    RShiftEqual,
    #[token("&=")]
    AmpersandEqual,
    #[token("^=")]
    CaretEqual,
    #[token("|=")]
    BarEqual,
    #[token(",")]
    Comma,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    //------------------- Whitespace -------------------
    #[regex("[ \r\n]")]
    Newline,
    #[regex("[ \t\r\n]", logos::skip)]
    Whitespace,

    //------------------- Other -------------------
    #[error]
    Error,
}
