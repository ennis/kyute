//! Parser utilities for box styles.
use crate::{css::parse_css_length_percentage, EnvKey, LengthOrPercentage};
use cssparser::{ParseError, Parser, Token};
use kyute::Environment;
use kyute_common::Atom;
use std::f32::consts::PI;

////////////////////////////////////////////////////////////////////////////////////////////////////
// Color
////////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    /// Direct color value.
    Value(crate::Color),
    /// Color in environment,
    Env(Atom),
}

impl Default for Color {
    fn default() -> Self {
        Color::Value(crate::Color::default())
    }
}

impl Color {
    pub fn compute(&self, env: &Environment) -> crate::Color {
        match *self {
            Color::Value(value) => value,
            Color::Env(ref var_name) => {
                let color = env.get_by_name::<crate::Color, _>(var_name.clone());
                if let Some(color) = color {
                    color
                } else {
                    warn!("invalid color reference: {}", var_name);
                    crate::Color::default()
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// colors
////////////////////////////////////////////////////////////////////////////////////////////////////

fn alpha<'i>(input: &mut Parser<'i, '_>) -> Result<f32, ParseError<'i, ()>> {
    if !input.is_exhausted() {
        input.expect_delim('/')?;
        let location = input.current_source_location();
        let alpha = match input.next()? {
            Token::Number { value, .. } => *value / 255.0,
            Token::Percentage { unit_value, .. } => *unit_value,
            t => return Err(location.new_unexpected_token_error(t.clone())),
        };
        Ok(alpha)
    } else {
        Ok(1.0)
    }
}

fn rgb_color<'i>(input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, ()>> {
    let location = input.current_source_location();
    let (r, is_number) = match input.next()? {
        Token::Number { value, .. } => (*value / 255.0, true),
        Token::Percentage { unit_value, .. } => (*unit_value, false),
        t => return Err(location.new_unexpected_token_error(t.clone())),
    };

    let g;
    let b;
    if is_number {
        g = input.expect_number()? / 255.0;
        b = input.expect_number()? / 255.0;
    } else {
        g = input.expect_percentage()?;
        b = input.expect_percentage()?;
    }

    let alpha = alpha(input)?;
    input.expect_exhausted()?;

    Ok(Color::Value(crate::Color::new(r, g, b, alpha)))
}

fn hsl_color<'i>(input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, ()>> {
    let location = input.current_source_location();
    let hue_degrees = match input.next()? {
        Token::Number { value, .. } => *value,
        Token::Dimension { value, unit, .. } => match &**unit {
            "deg" => *value,
            "grad" => *value * 360. / 400.,
            "rad" => *value * 360. / (2. * PI),
            "turn" => *value * 360.,
            _ => return Err(location.new_unexpected_token_error(Token::Ident(unit.clone()))),
        },
        t => return Err(location.new_unexpected_token_error(t.clone())),
    };

    let saturation = input.expect_percentage()?;
    let brightness = input.expect_percentage()?;
    let alpha = alpha(input)?;
    input.expect_exhausted()?;
    Ok(Color::Value(crate::Color::hsla(
        hue_degrees,
        saturation,
        brightness,
        alpha,
    )))
}

fn color_function<'i>(name: &str, input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, ()>> {
    let location = input.current_source_location();
    match name {
        "rgb" => rgb_color(input),
        "hsl" => hsl_color(input),
        _ => Err(location.new_unexpected_token_error(Token::Ident(name.to_owned().into()))),
    }
}

pub(crate) fn css_color<'i>(input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, ()>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Function(ref name) => {
            let name = name.clone();
            input.parse_nested_block(|input| {
                let color = color_function(&*name, input)?;
                Ok(color)
            })
        }
        t @ Token::Delim('$') => {
            // color from environment
            let var_name = input.expect_ident()?;
            Ok(Color::Env(Atom::from(&**var_name)))
        }
        t @ Token::Hash(ref digits) | t @ Token::IDHash(ref digits) => match crate::Color::try_from_hex(digits) {
            Ok(color) => Ok(Color::Value(color)),
            Err(_) => Err(location.new_unexpected_token_error(t.clone())),
        },
        t => Err(location.new_unexpected_token_error(t.clone())),
    }
}
