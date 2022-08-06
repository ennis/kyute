//! CSS parsing utilities.
use crate::{Length, LengthOrPercentage, UnitExt};
use cssparser::{ParseError, Parser, ParserInput, Token};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Utilities
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn parse_from_str<'i, T, F, E>(css: &'i str, f: F) -> Result<T, ParseError<'i, E>>
where
    F: for<'tt> FnOnce(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>,
{
    let mut input = ParserInput::new(css);
    let mut input = Parser::new(&mut input);
    input.parse_entirely(f)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// lengths
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn parse_css_length<'i>(input: &mut Parser<'i, '_>) -> Result<Length, ParseError<'i, ()>> {
    match input.next()? {
        token @ Token::Dimension { value, unit, .. } => {
            // be consistent with CSS and interpret px as DIPs; use "ppx" for physical pixels
            match &**unit {
                "px" => Ok((*value).dip()),
                "in" => Ok((*value).inch()),
                "pt" => Ok((*value).pt()),
                "ppx" => Ok((*value).px()),
                _ => {
                    let token = token.clone();
                    return Err(input.new_unexpected_token_error(token));
                }
            }
        }
        Token::Number { int_value: Some(0), .. } => Ok(Length::Dip(0.0)),
        token => {
            let token = token.clone();
            return Err(input.new_unexpected_token_error(token));
        }
    }
}

pub(crate) fn parse_css_length_percentage<'i>(
    input: &mut Parser<'i, '_>,
) -> Result<LengthOrPercentage, ParseError<'i, ()>> {
    if let Ok(length) = input.try_parse(parse_css_length) {
        Ok(LengthOrPercentage::Length(length))
    } else {
        Ok(LengthOrPercentage::Percentage(input.expect_percentage()? as f64))
    }
}
