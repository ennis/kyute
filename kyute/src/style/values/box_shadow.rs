use crate::{
    style::{
        values::{color::css_color, length::length},
        StyleCtx, ToComputedValue,
    },
    Color, Length,
};
use cssparser::{ParseError, Parser};

/// Box shadow parameters.
#[derive(Copy, Clone, Debug, PartialEq, serde::Deserialize)]
pub struct BoxShadow {
    pub color: Color,
    pub x_offset: Length,
    pub y_offset: Length,
    pub blur: Length,
    pub spread: Length,
    pub inset: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct ComputedBoxShadow {
    pub color: Color,
    pub x_offset: f64,
    pub y_offset: f64,
    pub blur: f64,
    pub spread: f64,
    pub inset: bool,
}

impl ToComputedValue for BoxShadow {
    type ComputedValue = ComputedBoxShadow;

    fn to_computed_value(&self, context: &StyleCtx) -> Self::ComputedValue {
        ComputedBoxShadow {
            color: self.color.clone(),
            x_offset: self.x_offset.to_computed_value(context),
            y_offset: self.y_offset.to_computed_value(context),
            blur: self.blur.to_computed_value(context),
            spread: self.spread.to_computed_value(context),
            inset: self.inset,
        }
    }
}

/// Array of box shadows.
///
/// The value of a CSS `box-shadow` property.
pub type BoxShadows = Vec<BoxShadow>;

/// Array of computed box shadows.
pub type ComputedBoxShadows = Vec<ComputedBoxShadow>;

impl ToComputedValue for BoxShadows {
    type ComputedValue = ComputedBoxShadows;

    fn to_computed_value(&self, context: &StyleCtx) -> ComputedBoxShadows {
        self.iter().map(|shadow| shadow.to_computed_value(context)).collect()
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
                    let x_offset = length(input)?;
                    let y_offset = length(input)?;
                    let blur = input.try_parse(length).unwrap_or(Length::zero());
                    let spread = input.try_parse(length).unwrap_or(Length::zero());
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
