use crate::{
    css::{parse_css_length, parse_from_str},
    drawing,
    style::utils::css_color,
    Color, LayoutConstraints, Length, Offset,
};
use cssparser::{ParseError, Parser};

/// Box shadow parameters.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serializing", derive(serde::Deserialize))]
pub struct BoxShadow {
    pub color: Color,
    pub x_offset: Length,
    pub y_offset: Length,
    pub blur: Length,
    pub spread: Length,
    pub inset: bool,
}

impl BoxShadow {
    pub(crate) fn compute(&self, constraints: &LayoutConstraints) -> drawing::BoxShadow {
        drawing::BoxShadow {
            color: self.color.clone(),
            offset: Offset::new(self.x_offset.compute(constraints), self.y_offset.compute(constraints)),
            blur: self.blur.compute(constraints),
            spread: self.spread.compute(constraints),
            inset: self.inset,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// box-shadow declaration
////////////////////////////////////////////////////////////////////////////////////////////////////

impl BoxShadow {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<BoxShadow, ParseError<'i, ()>> {
        let mut inset = false;
        let mut lengths = None;
        let mut color = None;

        loop {
            if !inset {
                if input.try_parse(|i| i.expect_ident_matching("inset")).is_ok() {
                    inset = true;
                    continue;
                }
            }

            if lengths.is_none() {
                let values = input.try_parse::<_, _, ParseError<'i, ()>>(|input| {
                    let x_offset = parse_css_length(input)?;
                    let y_offset = parse_css_length(input)?;
                    let blur = input.try_parse(parse_css_length).unwrap_or(Length::zero());
                    let spread = input.try_parse(parse_css_length).unwrap_or(Length::zero());
                    Ok((x_offset, y_offset, blur, spread))
                });

                if let Ok(values) = values {
                    lengths = Some(values);
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

        let lengths = lengths.ok_or(input.new_custom_error(()))?;
        Ok(BoxShadow {
            color: color.unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0)),
            x_offset: lengths.0,
            y_offset: lengths.1,
            blur: lengths.2,
            spread: lengths.3,
            inset,
        })
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

/// Array of box shadows.
///
/// The value of a CSS `box-shadow` property.
pub type BoxShadows = Vec<BoxShadow>;

pub(crate) fn parse_box_shadows<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<BoxShadow>, ParseError<'i, ()>> {
    if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
        return Ok(vec![]);
    }
    input.parse_comma_separated(BoxShadow::parse_impl)
}
