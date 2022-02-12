pub use bumpalo::Bump as Arena;
use lalrpop_util::lalrpop_mod;
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;

mod ast;
mod color;
lalrpop_mod!(pub grammar); // synthesized by LALRPOP

pub use ast::*;

pub type ParseError<'input> =
    lalrpop_util::ParseError<usize, lalrpop_util::lexer::Token<'input>, &'static str>;

pub struct ParseState<'ast> {
    arena: &'ast Arena,
}

impl<'ast> ParseState<'ast> {
    pub fn new(arena: &'ast Arena) -> ParseState<'ast> {
        ParseState { arena }
    }
}
