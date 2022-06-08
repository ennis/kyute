//! Parser for box styles.
use cssparser::{CowRcStr, ParseError, Parser, ParserInput, SourceLocation, Token};
use kyute_common::{Angle, Color, Length, UnitExt};
use std::f32::consts::PI;
/*use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while_m_n},
    character::{
        complete::{alpha1, alphanumeric1, char, digit1, space0, space1},
        is_hex_digit,
    },
    combinator::{eof, map, map_res, opt, peek, recognize},
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::{count, many0_count, many1, many_m_n, separated_list1},
    number::complete::double,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};*/
use crate::{
    style::{ColorStop, LinearGradient, Paint},
    widget::grid::Line,
};
use palette::{Hsla, RgbHue};
use std::str::FromStr;

/// Length units.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Unit {
    /// Pixels (physical).
    Px,
    /// Points.
    Pt,
    /// Inches.
    In,
    /// Device-independent pixels.
    Dip,
    /// Percentage of the parent widget's size.
    Percent,
}

/// Angle units
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AngleUnit {
    Turn,
    Radians,
    Degrees,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Utilities
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn integer_i32(input: &str) -> IResult<&str, i32> {
    map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| s.parse::<i32>())(input)
}

fn integer_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>())(input)
}

fn integer_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

/// All length units
fn length_unit(input: &str) -> IResult<&str, Unit> {
    // px, dip, etc.
    map(
        alt((tag("px"), tag("dip"), tag("in"), tag("pt"), tag("%"))),
        |s: &str| match s {
            "px" => Unit::Px,
            "dip" => Unit::Dip,
            "%" => Unit::Percent,
            "in" => Unit::In,
            "pt" => Unit::Pt,
            _ => unreachable!(),
        },
    )(input)
}

/// All angle units
fn angle_unit(input: &str) -> IResult<&str, AngleUnit> {
    // px, dip, etc.
    map(alt((tag("turn"), tag("rad"), tag("deg"))), |s: &str| match s {
        "turn" => AngleUnit::Turn,
        "rad" => AngleUnit::Radians,
        "deg" => AngleUnit::Degrees,
        _ => unreachable!(),
    })(input)
}

/// Percentage value
fn percentage(input: &str) -> IResult<&str, f64> {
    terminated(double, '%')(input)
}

/// Percentage value
fn normalized_percentage(input: &str) -> IResult<&str, f64> {
    map(percentage, |x| x / 100.0)(input)
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////
// colors
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn hex_color(input: &str) -> IResult<&str, Color> {
    map(
        recognize(preceded(tag('#'), take_while(is_hex_digit))),
        |color_str| match Color::try_from_hex(color_str) {
            Ok(color) => color,
            Err(_) => {
                warn!("invalid hex color: {}", color_str);
                Color::default()
            }
        },
    )(input)
}

// "/ <alpha-value>"
fn alpha_suffix(input: &str) -> IResult<&str, f64> {
    preceded(
        delimited(space0, char('/'), space0),
        alt((normalized_percentage, double)),
    )(input)
}

// rgb(R G B [/ A]?)
fn rgb_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("rgb(")(input)?;
    let (input, (r, g, b)) = alt((
        tuple((
            preceded(space0, normalized_percentage),
            preceded(space0, normalized_percentage),
            preceded(space0, normalized_percentage),
        )),
        tuple((
            map(preceded(space0, double), |x| x / 255.0),
            map(preceded(space0, double), |x| x / 255.0),
            map(preceded(space0, double), |x| x / 255.0),
        )),
    ))(input)?;

    let (input, alpha) = opt(alpha_suffix)(input)?;
    let (input, _) = preceded(space0, char(')'))(input)?;

    let color = if let Some(alpha) = alpha {
        Color::new(r as f32, g as f32, b as f32, alpha as f32)
    } else {
        Color::new(r as f32, g as f32, b as f32, 1.0)
    };

    Ok((input, color))
}

// hsl(...)
fn hsl_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("hsl(")(input)?;

    let (input, (hue, hue_unit)) = preceded(space0, tuple((double, opt(angle_unit))))(input)?;
    let (input, saturation) = preceded(space0, normalized_percentage)(input)?;
    let (input, lightness) = preceded(space0, normalized_percentage)(input)?;
    let (input, alpha) = opt(alpha_suffix)(input)?;
    let (input, _) = preceded(space0, char(')'))(input)?;

    let hue = match hue_unit {
        Some(AngleUnit::Degrees) | None => hue,
        Some(AngleUnit::Radians) => hue / std::f64::consts::PI * 180.0,
        Some(AngleUnit::Turn) => hue * 360.0,
    };

    let color = Color::hsla(hue, saturation as f32, lightness as f32, alpha.unwrap_or(1.0) as f32);
    Ok((input, color))
}

fn color(input: &str) -> IResult<&str, Color> {
    alt((hex_color, rgb_color, hsl_color))(input)
}*/

/*
fn component_number<'i>(input: &mut Parser<'i, '_>) -> Result<f64, ParseError<'i, ()>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Number { value, .. } => Ok(*value as f64 / 255.0),
        t => Err(location.new_unexpected_token_error(t.clone())),
    }
}

fn component_percentage<'i>(input: &mut Parser<'i, '_>) -> Result<f64, ParseError<'i, ()>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Number { value, .. } => Ok(*value as f64 / 255.0),
        t => Err(location.new_unexpected_token_error(t.clone())),
    }
}*/

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
        g = input.expect_number()?;
        b = input.expect_number()?;
    } else {
        g = input.expect_percentage()?;
        b = input.expect_percentage()?;
    }

    let alpha = alpha(input)?;
    input.expect_exhausted()?;

    Ok(Color::new(r, g, b, alpha))
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
    Ok(Color::hsla(hue_degrees, saturation, brightness, alpha))
}

fn color_function<'i>(name: &str, input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, ()>> {
    let location = input.current_source_location();
    match name {
        "rgb" => rgb_color(input),
        "hsl" => hsl_color(input),
        _ => Err(location.new_unexpected_token_error(Token::Ident(name.to_owned().into()))),
    }
}

fn color<'i>(input: &mut Parser<'i, '_>) -> Result<Color, ParseError<'i, ()>> {
    let location = input.current_source_location();
    match input.next()? {
        Token::Function(ref name) => {
            let name = name.clone();
            input.parse_nested_block(|input| {
                let color = color_function(&*name, input)?;
                Ok(color)
            })
        }
        t @ Token::Hash(ref digits) | t @ Token::IDHash(ref digits) => match Color::try_from_hex(digits) {
            Ok(color) => Ok(color),
            Err(_) => Err(location.new_unexpected_token_error(t.clone())),
        },
        t => Err(location.new_unexpected_token_error(t.clone())),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// linear_gradient
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn angle(value: f32, angle_unit: &str) -> Option<f32> {

}*/

enum BoxSide {
    Bottom,
    Top,
    Left,
    Right,
}

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

fn box_side<'i>(input: &mut Parser<'i, '_>) -> Result<BoxSide, ParseError<'i, ()>> {
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

#[derive(Copy, Clone, Debug, PartialEq)]
struct LineDirection {
    angle: f32,
}

fn line_direction<'i>(input: &mut Parser<'i, '_>) -> Result<LineDirection, ParseError<'i, ()>> {
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

fn linear_color_stop<'i>(input: &mut Parser<'i, '_>) -> Result<ColorStop, ParseError<'i, ()>> {
    let color = color(input)?;
    let position = input.try_parse(Parser::expect_percentage).ok();
    Ok(ColorStop {
        color,
        position: position.map(|x| x as f64),
    })
}

#[derive(Clone, Debug, PartialEq)]
struct CssLinearGradient {
    direction: LineDirection,
    stops: Vec<ColorStop>,
}

fn linear_gradient<'i>(input: &mut Parser<'i, '_>) -> Result<CssLinearGradient, ParseError<'i, ()>> {
    input.expect_function_matching("linear-gradient")?;
    input.parse_nested_block(|input| {
        let direction = if let Some(line_direction) = input.try_parse(line_direction).ok() {
            input.expect_comma()?;
            line_direction
        } else {
            LineDirection { angle: 180.0 }
        };

        let mut stops = Vec::new();
        stops.push(linear_color_stop(input)?);
        while !input.is_exhausted() {
            input.expect_comma()?;
            stops.push(linear_color_stop(input)?);
        }

        Ok(CssLinearGradient { direction, stops })
    })
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// background
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Paint {
    pub fn parse(css: &str) -> Result<Paint, ()> {
        let mut input = ParserInput::new(css);
        let mut input = Parser::new(&mut input);

        if let Ok(color) = input.try_parse(color) {
            Ok(Paint::SolidColor { color })
        } else if let Ok(linear_gradient) = input.try_parse(linear_gradient) {
            Ok(Paint::LinearGradient(LinearGradient {
                angle: Angle::degrees(linear_gradient.direction.angle as f64),
                stops: linear_gradient.stops,
            }))
        } else {
            warn!("invalid paint value: `{}`", css);
            Err(())
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::style::{
        parser::{linear_gradient, CssLinearGradient, LineDirection},
        ColorStop,
    };
    use cssparser::{Parser, ParserInput};
    use kyute_common::Color;

    fn parse_string<'i, T: 'i, E: 'i>(
        input: &'i str,
        f: impl FnOnce(&mut Parser<'i, '_>) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut input = ParserInput::new(input);
        let mut input = Parser::new(&mut input);
        f(&mut input)
    }

    #[test]
    fn test_linear_gradient() {
        assert_eq!(
            parse_string("linear-gradient(#D7D5D7, #F6F5F6)", linear_gradient),
            Ok(CssLinearGradient {
                direction: LineDirection { angle: 180.0 },
                stops: vec![
                    ColorStop {
                        position: None,
                        color: Color::from_hex("#D7D5D7")
                    },
                    ColorStop {
                        position: None,
                        color: Color::from_hex("#F6F5F6")
                    },
                ]
            })
        );
    }
}
