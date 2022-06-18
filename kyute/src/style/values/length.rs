//! Length parsers

use crate::{Length, UnitExt};
use cssparser::{ParseError, Parser, Token};

/// Parses a CSS length.
pub(crate) fn length<'i>(input: &mut Parser<'i, '_>) -> Result<Length, ParseError<'i, ()>> {
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

/// Parses a CSS length or percentage.
pub(crate) fn length_percentage<'i>(input: &mut Parser<'i, '_>) -> Result<Length, ParseError<'i, ()>> {
    if let Ok(length) = input.try_parse(length) {
        Ok(length)
    } else {
        Ok(Length::Proportional(input.expect_percentage()? as f64))
    }
}
