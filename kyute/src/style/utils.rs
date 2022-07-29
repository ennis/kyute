//! Parser utilities for box styles.
use crate::{css::parse_css_length_percentage, Color, LengthOrPercentage};
use cssparser::{ParseError, Parser, Token};
use std::f32::consts::PI;

////////////////////////////////////////////////////////////////////////////////////////////////////
// padding
////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) fn padding<'i>(input: &mut Parser<'i, '_>) -> Result<[LengthOrPercentage; 4], ParseError<'i, ()>> {
    let length1 = parse_css_length_percentage(input)?;
    let length2 = input.try_parse(parse_css_length_percentage).ok();
    let length3 = input.try_parse(parse_css_length_percentage).ok();
    let length4 = input.try_parse(parse_css_length_percentage).ok();

    let padding = match (length1, length2, length3, length4) {
        (padding, None, None, None) => [padding; 4],
        (top_and_bottom, Some(left_and_right), None, None) => {
            [top_and_bottom, left_and_right, top_and_bottom, left_and_right]
        }
        (top, Some(right_and_left), Some(bottom), None) => [top, right_and_left, bottom, right_and_left],
        (top, Some(right), Some(bottom), Some(left)) => [top, right, bottom, left],
        _ => unreachable!(),
    };
    Ok(padding)
}
