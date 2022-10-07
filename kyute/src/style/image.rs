//! Description of paints.
use crate::{css::parse_from_str, drawing, style, style::color::css_color, Color, EnvKey, Environment, UnitExt};
use cssparser::{ParseError, Parser, Token};
use kyute_common::Angle;
use std::{convert::TryFrom, f32::consts::PI};

////////////////////////////////////////////////////////////////////////////////////////////////////
// CSS image
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Represents a gradient stop.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorStop {
    /// Position of the stop along the gradient segment, normalized between zero and one.
    ///
    /// If `None`, the position is inferred from the position of the surrounding stops.
    pub position: Option<f64>,
    /// Stop color.
    pub color: style::Color,
}

/// Describes a linear color gradient.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Direction of the gradient line.
    pub angle: Angle,
    /// List of color stops.
    pub stops: Vec<ColorStop>,
}

impl LinearGradient {
    pub fn compute(&self, env: &Environment) -> drawing::LinearGradient {
        drawing::LinearGradient {
            angle: self.angle,
            stops: self
                .stops
                .iter()
                .map(|stop| drawing::ColorStop {
                    position: stop.position,
                    color: stop.color.compute(env),
                })
                .collect(),
        }
    }
}

/// Value of the background property.
#[derive(Clone, Debug)]
pub enum Image {
    Color(style::Color),
    LinearGradient(LinearGradient),
}

impl Default for Image {
    fn default() -> Self {
        Image::Color(Default::default())
    }
}

impl Image {
    pub fn compute_paint(&self, env: &Environment) -> drawing::Paint {
        match self {
            Image::Color(color) => drawing::Paint::Color(color.compute(env)),
            Image::LinearGradient(gradient) => drawing::Paint::LinearGradient(gradient.compute(env)),
        }
    }
}

impl From<Color> for Image {
    fn from(color: Color) -> Self {
        Image::Color(style::Color::Value(color))
    }
}

impl From<EnvKey<Color>> for Image {
    fn from(color: EnvKey<Color>) -> Self {
        Image::Color(style::Color::Env(color.atom()))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// parser
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Image {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Image, ParseError<'i, ()>> {
        if let Ok(color) = input.try_parse(css_color) {
            Ok(Image::Color(color))
        } else if let Ok(linear_gradient) = input.try_parse(linear_gradient) {
            Ok(Image::LinearGradient(linear_gradient))
        } else {
            Err(input.new_custom_error(()))
        }
    }

    pub(crate) fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

impl<'a> TryFrom<&'a str> for Image {
    type Error = ParseError<'a, ()>;
    fn try_from(css: &'a str) -> Result<Self, Self::Error> {
        Self::parse(css)
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
        let side_1 = BoxSide::parse(input)?;
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

fn color_stop<'i>(input: &mut Parser<'i, '_>) -> Result<ColorStop, ParseError<'i, ()>> {
    let color = css_color(input)?;
    let position = input.try_parse(Parser::expect_percentage).ok();
    Ok(ColorStop {
        color,
        position: position.map(|x| x as f64),
    })
}

fn linear_gradient<'i>(input: &mut Parser<'i, '_>) -> Result<LinearGradient, ParseError<'i, ()>> {
    input.expect_function_matching("linear-gradient")?;
    input.parse_nested_block(|input| {
        let direction = if let Some(line_direction) = input.try_parse(LineDirection::parse).ok() {
            input.expect_comma()?;
            line_direction
        } else {
            LineDirection { angle: 180.0 }
        };

        let mut stops = Vec::new();
        stops.push(color_stop(input)?);
        while !input.is_exhausted() {
            input.expect_comma()?;
            stops.push(color_stop(input)?);
        }

        Ok(LinearGradient {
            angle: direction.angle.degrees(),
            stops,
        })
    })
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Vector drawables
////////////////////////////////////////////////////////////////////////////////////////////////////
