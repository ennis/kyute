//! Border description.
use crate::{
    css::{parse_css_length, parse_from_str},
    drawing,
    style::utils::css_color,
    Color, Length, UnitExt,
};
use cssparser::{ParseError, Parser, Token};
use std::convert::TryFrom;

/// CSS border shorthand.
#[derive(Clone, Debug)]
pub struct Border {
    /// Left,top,right,bottom border widths.
    pub widths: [Length; 4],
    pub color: Color,
    pub line_style: drawing::BorderStyle,
}

impl Default for Border {
    fn default() -> Self {
        Border {
            widths: [Length::zero(); 4],
            color: Color::default(),
            line_style: Default::default(),
        }
    }
}

impl Border {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Border, ParseError<'i, ()>> {
        let mut line_width = None;
        let mut line_style = None;
        let mut color = None;

        loop {
            if line_width.is_none() {
                let width = input.try_parse(|input| {
                    if input.try_parse(|i| i.expect_ident_matching("thin")).is_ok() {
                        Ok(1.dip())
                    } else if input.try_parse(|i| i.expect_ident_matching("medium")).is_ok() {
                        Ok(2.dip())
                    } else if input.try_parse(|i| i.expect_ident_matching("thick")).is_ok() {
                        Ok(3.dip())
                    } else {
                        input.try_parse(parse_css_length)
                    }
                });

                if let Ok(width) = width {
                    line_width = Some(width);
                    continue;
                }
            }

            if line_style.is_none() {
                let style = input.try_parse::<_, _, ParseError<'i, ()>>(|input| match input.next()? {
                    Token::Ident(ident) if &**ident == "solid" => Ok(drawing::BorderStyle::Solid),
                    Token::Ident(ident) if &**ident == "dotted" => Ok(drawing::BorderStyle::Dotted),
                    token => {
                        let token = token.clone();
                        Err(input.new_unexpected_token_error(token))
                    }
                });

                if let Ok(style) = style {
                    line_style = Some(style);
                    continue;
                }
            }

            if color.is_none() {
                if let Ok(c) = input.try_parse(css_color) {
                    color = Some(c);
                    continue;
                }
            }

            break;
        }

        if line_width.is_none() && line_style.is_none() && color.is_none() {
            return Err(input.new_custom_error(()));
        }

        let line_width = line_width.unwrap_or(Length::zero());

        Ok(Border {
            widths: [line_width; 4],
            color: color.unwrap_or_default(),
            line_style: line_style.unwrap_or_default(),
        })
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

impl<'a> TryFrom<&'a str> for Border {
    type Error = ParseError<'a, ()>;
    fn try_from(css: &'a str) -> Result<Self, Self::Error> {
        Self::parse(css)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// border-radius
////////////////////////////////////////////////////////////////////////////////////////////////////

/// border-radius
pub(crate) fn border_radius<'i>(input: &mut Parser<'i, '_>) -> Result<[Length; 4], ParseError<'i, ()>> {
    // <length-percentage>{1,4} [ / <length-percentage>{1,4} ]?
    // (but we don't support the '/' part, yet.)

    let length1 = parse_css_length(input)?;
    let length2 = input.try_parse(parse_css_length).ok();
    let length3 = input.try_parse(parse_css_length).ok();
    let length4 = input.try_parse(parse_css_length).ok();

    let radii = match (length1, length2, length3, length4) {
        (radius, None, None, None) => [radius; 4],
        (top_left_and_bottom_right, Some(top_right_and_bottom_left), None, None) => [
            top_left_and_bottom_right,
            top_right_and_bottom_left,
            top_left_and_bottom_right,
            top_right_and_bottom_left,
        ],
        (top_left, Some(top_right_and_bottom_left), Some(bottom_right), None) => [
            top_left,
            top_right_and_bottom_left,
            bottom_right,
            top_right_and_bottom_left,
        ],
        (top_left, Some(top_right), Some(bottom_right), Some(bottom_left)) => {
            [top_left, top_right, bottom_right, bottom_left]
        }
        _ => unreachable!(),
    };
    Ok(radii)
}
