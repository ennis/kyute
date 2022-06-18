//! CSS parser utilities

use cssparser::{ParseError, Parser, ParserInput};

pub(crate) fn parse_from_str<'i, T, F, E>(css: &'i str, f: F) -> Result<T, ParseError<'i, E>>
where
    F: for<'tt> FnOnce(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>,
{
    let mut input = ParserInput::new(css);
    let mut input = Parser::new(&mut input);
    input.parse_entirely(f)
}

/// Parses the remainder of a property after the colon and eat the semicolon after.
pub(crate) fn parse_property_remainder<'i, T, F, E>(input: &mut Parser<'i, '_>, f: F) -> Result<T, ParseError<'i, E>>
where
    F: for<'tt> FnOnce(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>,
{
    input.parse_until_after(cssparser::Delimiter::Semicolon, f)
}
