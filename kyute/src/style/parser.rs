//! Parser for box styles.
use crate::{
    style::{BlendMode, Border, BorderStyle, BoxShadow, ColorStop, LinearGradient, Paint, Style},
    widget::grid::Line,
    Angle, Color, Length, UnitExt,
};
use cssparser::{BasicParseErrorKind, CowRcStr, Delimiters, ParseError, Parser, ParserInput, SourceLocation, Token};
use palette::{Hsla, RgbHue};
use std::{f32::consts::PI, str::FromStr};

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

pub(crate) fn parse_property_remainder<'i, T, F, E>(input: &mut Parser<'i, '_>, f: F) -> Result<T, ParseError<'i, E>>
where
    F: for<'tt> FnOnce(&mut Parser<'i, 'tt>) -> Result<T, ParseError<'i, E>>,
{
    input.parse_until_after(cssparser::Delimiter::Semicolon, f)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// lengths
////////////////////////////////////////////////////////////////////////////////////////////////////

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

pub(crate) fn length_percentage<'i>(input: &mut Parser<'i, '_>) -> Result<Length, ParseError<'i, ()>> {
    if let Ok(length) = input.try_parse(length) {
        Ok(length)
    } else {
        Ok(Length::Proportional(input.expect_percentage()? as f64))
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// background
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Paint {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Paint, ParseError<'i, ()>> {
        if let Ok(color) = input.try_parse(css_color) {
            Ok(Paint::SolidColor(color))
        } else if let Ok(linear_gradient) = input.try_parse(LinearGradient::parse_impl) {
            Ok(Paint::LinearGradient(linear_gradient))
        } else {
            Err(input.new_custom_error(()))
        }
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// box-shadow
////////////////////////////////////////////////////////////////////////////////////////////////////

impl BoxShadow {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<BoxShadow, ParseError<'i, ()>> {
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// border
////////////////////////////////////////////////////////////////////////////////////////////////////

impl Border {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Border, ParseError<'i, ()>> {
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
                        input.try_parse(length)
                    }
                });

                if let Ok(width) = width {
                    line_width = Some(width);
                    continue;
                }
            }

            if line_style.is_none() {
                let style = input.try_parse::<_, _, ParseError<'i, ()>>(|input| match input.next()? {
                    Token::Ident(ident) if &**ident == "solid" => Ok(BorderStyle::Solid),
                    Token::Ident(ident) if &**ident == "dotted" => Ok(BorderStyle::Dotted),
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
            paint: color.map(Paint::SolidColor).unwrap_or_default(),
            line_style: line_style.unwrap_or_default(),
            blend_mode: BlendMode::SrcOver,
        })
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

/// border-radius
fn border_radius<'i>(input: &mut Parser<'i, '_>) -> Result<[Length; 4], ParseError<'i, ()>> {
    // <length-percentage>{1,4} [ / <length-percentage>{1,4} ]?
    // (but we don't support the '/' part, yet.)

    let length1 = length_percentage(input)?;
    let length2 = input.try_parse(length_percentage).ok();
    let length3 = input.try_parse(length_percentage).ok();
    let length4 = input.try_parse(length_percentage).ok();

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Style
////////////////////////////////////////////////////////////////////////////////////////////////////
impl Style {
    fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Style, ParseError<'i, ()>> {
        let mut style = Style {
            border_radii: [Length::zero(); 4],
            border: None,
            background: None,
            box_shadows: vec![],
        };

        // CSS inline style parser
        while !input.is_exhausted() {
            let prop_name = input.expect_ident()?.clone();
            input.expect_colon()?;
            match &*prop_name {
                "background" => {
                    style.background = Some(parse_property_remainder(input, Paint::parse_impl)?);
                }
                "border" => {
                    style.border = Some(parse_property_remainder(input, Border::parse_impl)?);
                }
                "border-radius" => {
                    style.border_radii = parse_property_remainder(input, border_radius)?;
                }
                "box-shadow" => {
                    style.box_shadows =
                        parse_property_remainder(input, |input| input.parse_comma_separated(BoxShadow::parse_impl))?;
                }
                _ => {
                    // unrecognized property
                    return Err(input.new_custom_error(()));
                }
            }
        }

        Ok(style)
    }

    pub fn parse(css: &str) -> Result<Self, ParseError<()>> {
        parse_from_str(css, Self::parse_impl)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::style::{
        parser::{linear_gradient, LineDirection},
        ColorStop, LinearGradient,
    };
    use cssparser::{Parser, ParserInput};
    use kyute_common::{Color, UnitExt};

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
            Ok(LinearGradient {
                angle: 180.degrees(),
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
