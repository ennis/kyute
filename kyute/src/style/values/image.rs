//! Description of paints.
use crate::{cache, style::values::color::css_color, Angle, Color, Data, Offset, Rect, UnitExt};
use cssparser::{ParseError, Parser, Token};
use std::{convert::TryFrom, f32::consts::PI, ffi::c_void, fmt, mem};

////////////////////////////////////////////////////////////////////////////////////////////////////
// CSS image
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents a gradient stop.
#[derive(Clone, Debug, Data, PartialEq, serde::Deserialize)]
pub struct ColorStop {
    /// Position of the stop along the gradient segment, normalized between zero and one.
    ///
    /// If `None`, the position is inferred from the position of the surrounding stops.
    pub position: Option<f64>,
    /// Stop color.
    pub color: Color,
}

/// Describes a linear color gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Direction of the gradient line.
    pub angle: Angle,
    /// List of color stops.
    pub stops: Vec<ColorStop>,
}

impl Data for LinearGradient {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

/// Image repeat mode.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Data, serde::Deserialize)]
pub enum RepeatMode {
    Repeat,
    NoRepeat,
}

/// Value of the background property.
#[derive(Clone, Debug)]
pub enum Image {
    Color(Color),
    LinearGradient(LinearGradient),
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// parser
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Image {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Image, ParseError<'i, ()>> {
        if let Ok(color) = input.try_parse(css_color) {
            Ok(Image::Color(color))
        } else if let Ok(linear_gradient) = input.try_parse(LinearGradient::parse_impl) {
            Ok(Image::LinearGradient(linear_gradient))
        } else {
            Err(input.new_custom_error(()))
        }
    }

    pub(crate) fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

/// Parses an angle.
fn angle<'i>(input: &mut Parser<'i, '_>) -> Result<f32, ParseError<'i, ()>> {
    let location = input.current_source_location();
    let token = input.next()?;
    match token {
        Token::Dimension { value, unit, .. } => match &**unit {
            "deg" => Ok(*value),
            "grad" => Ok(*value * 360. / 400.),
            "rad" => Ok(*value * 360. / (2. * PI)),
            "turn" => Ok(*value * 360.),
            _ => return Err(location.new_unexpected_token_error(token.clone())),
        },
        _ => return Err(location.new_unexpected_token_error(token.clone())),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum BoxSide {
    Bottom,
    Top,
    Left,
    Right,
}

impl BoxSide {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<BoxSide, ParseError<'i, ()>> {
        let location = input.current_source_location();
        let ident = input.expect_ident()?;
        match &**ident {
            "left" => Ok(BoxSide::Left),
            "right" => Ok(BoxSide::Right),
            "top" => Ok(BoxSide::Top),
            "bottom" => Ok(BoxSide::Bottom),
            _ => Err(location.new_unexpected_token_error(Token::Ident(ident.clone()))),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct LineDirection {
    angle: f32,
}

impl LineDirection {
    fn parse<'i>(input: &mut Parser<'i, '_>) -> Result<LineDirection, ParseError<'i, ()>> {
        if let Ok(angle) = input.try_parse(angle) {
            return Ok(LineDirection { angle });
        }

        input.expect_ident_matching("to")?;
        let side_1 = box_side(input)?;
        // TODO
        //let side_2 = input.try_parse(box_side).ok();

        let angle = match side_1 {
            BoxSide::Top => 0.0,
            BoxSide::Right => 90.0,
            BoxSide::Bottom => 180.0,
            BoxSide::Left => 270.0,
        };

        Ok(LineDirection { angle })
    }
}

impl ColorStop {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<ColorStop, ParseError<'i, ()>> {
        let color = css_color(input)?;
        let position = input.try_parse(Parser::expect_percentage).ok();
        Ok(ColorStop {
            color,
            position: position.map(|x| x as f64),
        })
    }
}

impl LinearGradient {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<LinearGradient, ParseError<'i, ()>> {
        input.expect_function_matching("linear-gradient")?;
        input.parse_nested_block(|input| {
            let direction = if let Some(line_direction) = input.try_parse(line_direction).ok() {
                input.expect_comma()?;
                line_direction
            } else {
                LineDirection { angle: 180.0 }
            };

            let mut stops = Vec::new();
            stops.push(ColorStop::parse_impl(input)?);
            while !input.is_exhausted() {
                input.expect_comma()?;
                stops.push(ColorStop::parse_impl(input)?);
            }

            Ok(LinearGradient {
                angle: direction.angle.degrees(),
                stops,
            })
        })
    }
}

/*
impl LinearGradient {
    /// Creates a new `LinearGradient`, with no stops.
    pub fn new() -> LinearGradient {
        LinearGradient {
            angle: Default::default(),
            stops: vec![],
        }
    }

    /// Sets the gradient angle.
    pub fn angle(mut self, angle: Angle) -> Self {
        self.angle = angle;
        self
    }

    /// Appends a color stop to this gradient.
    pub fn stop(mut self, color: Color, position: impl Into<Option<f64>>) -> Self {
        self.stops.push(ColorStop {
            color,
            position: position.into(),
        });
        self
    }

    /// Resolves color stop positions.
    ///
    /// See https://www.w3.org/TR/css-images-3/#color-stop-fixup
    pub(crate) fn resolve_stop_positions(&mut self) {
        if self.stops.len() < 2 {
            warn!("invalid gradient (must have at least two stops)");
            return;
        }

        // CSS Images Module Level 3 - 3.4.3. Color Stop “Fixup”
        //
        //      If the first color stop does not have a position, set its position to 0%.
        //      If the last color stop does not have a position, set its position to 100%.
        //
        self.stops.first_mut().unwrap().position.get_or_insert(0.0);
        self.stops.last_mut().unwrap().position.get_or_insert(1.0);

        //
        //      If a color stop or transition hint has a position that is less than the specified position
        //      of any color stop or transition hint before it in the list, set its position to be equal
        //      to the largest specified position of any color stop or transition hint before it.
        //
        let mut cur_pos = self.stops.first().unwrap().position.unwrap();
        for stop in self.stops.iter_mut() {
            if let Some(mut pos) = stop.position {
                if pos < cur_pos {
                    pos = cur_pos;
                }
                cur_pos = pos;
            }
        }

        //
        //      If any color stop still does not have a position, then, for each run of adjacent color stops without positions,
        //      set their positions so that they are evenly spaced between the preceding and following color stops with positions.
        //
        let mut i = 0;
        while i < self.stops.len() {
            if self.stops[i].position.is_none() {
                let mut j = i + 1;
                while self.stops[j].position.is_none() {
                    j += 1;
                }
                let len = j - i + 1;
                let a = self.stops[i - 1].position.unwrap();
                let b = self.stops[j].position.unwrap();
                for k in i..j {
                    self.stops[i].position = Some(a + (b - a) * (k - i + 1) as f64 / len as f64);
                }
                i = j;
            } else {
                i += 1;
            }
        }
    }
}*/

impl Default for LinearGradient {
    fn default() -> Self {
        Self::new()
    }
}

/*
/// From CSS value.
impl TryFrom<&str> for Paint {
    type Error = ();
    fn try_from(css: &str) -> Result<Self, ()> {
        Paint::parse(css).map_err(|_| ())
    }
}
*/
