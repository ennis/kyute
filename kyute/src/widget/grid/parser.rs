//! Parser for the grid definition/placement language.

use crate::widget::{
    grid::{Area, GridTemplate, Line, LineRange, TrackItem},
    GridLength,
};
use kyute_common::{Length, UnitExt};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, digit1, space0, space1},
    combinator::{eof, map, map_res, opt, peek, recognize},
    error::{context, make_error, ErrorKind, ParseError, VerboseError},
    multi::{count, many0_count, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Finish, IResult,
};
use std::str::FromStr;

/// Track length units.
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
    /// Fraction of remaining flex space.
    Fractional,
}

fn parse_standalone<'a, T>(
    input: &'a str,
    parser: impl Fn(&'a str) -> IResult<&'a str, T>,
) -> Result<T, nom::error::Error<String>> {
    terminated(parser, eof)(input)
        .map_err(|e| e.to_owned())
        .finish()
        .map(|(_, value)| value)
}

fn integer_i32(input: &str) -> IResult<&str, i32> {
    map_res(recognize(pair(opt(char('-')), digit1)), |s:&str| s.parse::<i32>())(input)
}

fn integer_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>())(input)
}

fn integer_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

/// All units except 'fr'
fn non_fractional_unit(input: &str) -> IResult<&str, Unit> {
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

/// All grid track length units
fn unit(input: &str) -> IResult<&str, Unit> {
    alt((non_fractional_unit, map(tag("fr"), |_| Unit::Fractional)))(input)
}

/// Parses a length.
fn non_fractional_length(input: &str) -> IResult<&str, Length> {
    let (input, len) = integer_u32(input)?;
    let (input, unit) = opt(non_fractional_unit)(input)?;
    match unit {
        None => Ok((input, len.dip())),
        Some(Unit::Dip) => Ok((input, len.dip())),
        Some(Unit::Pt) => Ok((input, len.pt())),
        Some(Unit::In) => Ok((input, len.inch())),
        Some(Unit::Px) => Ok((input, len.px())),
        Some(Unit::Percent) => Ok((input, len.percent())),
        Some(Unit::Fractional) => unreachable!(),
    }
}

/// Parses a length.
fn length(input: &str) -> IResult<&str, GridLength> {
    let (input, len) = map_res(digit1, |s: &str| s.parse::<u32>())(input)?;
    let (input, unit) = opt(unit)(input)?;
    match unit {
        None => Ok((input, GridLength::Fixed(len.dip()))),
        Some(Unit::Dip) => Ok((input, GridLength::Fixed(len.dip()))),
        Some(Unit::Pt) => Ok((input, GridLength::Fixed(len.pt()))),
        Some(Unit::In) => Ok((input, GridLength::Fixed(len.inch()))),
        Some(Unit::Px) => Ok((input, GridLength::Fixed(len.px()))),
        Some(Unit::Percent) => Ok((input, GridLength::Fixed(len.percent()))),
        Some(Unit::Fractional) => Ok((input, GridLength::Flex(len as f64))),
    }
}

fn length_or_auto(input: &str) -> IResult<&str, GridLength> {
    context("length_or_auto", alt((map(tag("auto"), |_| GridLength::Auto), length)))(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    context(
        "identifier",
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_"), tag("-")))),
        )),
    )(input)
}

/// Track line tag.
fn line_tags(input: &str) -> IResult<&str, Vec<&str>> {
    context(
        "line_tags",
        delimited(char('['), separated_list1(space1, identifier), char(']')),
    )(input)
}

/// Implicit track definition
fn implicit_track(input: &str) -> IResult<&str, GridLength> {
    context("implicit_track", delimited(char('{'), length, char('}')))(input)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// TrackItem
////////////////////////////////////////////////////////////////////////////////////////////////////

fn track_item(input: &str) -> IResult<&str, TrackItem> {
    context(
        "track_item",
        alt((
            map(line_tags, TrackItem::LineTags),
            map(length_or_auto, TrackItem::Track),
            map(implicit_track, TrackItem::ImplicitTrack),
        )),
    )(input)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// GridTemplate
////////////////////////////////////////////////////////////////////////////////////////////////////

fn grid_template(input: &str) -> IResult<&str, GridTemplate> {
    let (input, _) = space0(input)?;
    let (input, rows) = separated_list1(space1, track_item)(input)?;
    let (input, _) = delimited(space0, char('/'), space0)(input)?;
    let (input, columns) = separated_list1(space1, track_item)(input)?;
    let (input, _) = space0(input)?;
    let (input, gaps) = opt(preceded(
        delimited(space0, char('/'), space0),
        tuple((non_fractional_length, opt(preceded(space0, non_fractional_length)))),
    ))(input)?;

    let spec = match gaps {
        None => GridTemplate {
            rows,
            columns,
            row_gap: None,
            column_gap: None,
        },
        Some((gap_1, None)) => GridTemplate {
            rows,
            columns,
            row_gap: Some(gap_1),
            column_gap: Some(gap_1),
        },
        Some((gap_1, Some(gap_2))) => GridTemplate {
            rows,
            columns,
            row_gap: Some(gap_1),
            column_gap: Some(gap_2),
        },
    };
    Ok((input, spec))
}

impl<'a> GridTemplate<'a> {
    pub fn parse(input: &'a str) -> Result<Self, nom::error::Error<String>> {
        parse_standalone(input, grid_template)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Line
////////////////////////////////////////////////////////////////////////////////////////////////////

fn line(input: &str) -> IResult<&str, Line> {
    alt((
        map(tag("auto"), |_| Line::Auto),
        map(preceded(tag("span"), preceded(space0, integer_usize)), Line::Span),
        map(identifier, Line::Named),
        map(integer_i32, Line::Index),
    ))(input)
}

impl<'a> Line<'a> {
    pub fn parse(input: &'a str) -> Result<Self, nom::error::Error<String>> {
        parse_standalone(input, line)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// LineRange
////////////////////////////////////////////////////////////////////////////////////////////////////

fn slash_sep(input: &str) -> IResult<&str, char> {
    delimited(space0, char('/'), space0)(input)
}

fn line_range(input: &str) -> IResult<&str, LineRange> {
    alt((
        map(separated_pair(line, slash_sep, line), |(start, end)| LineRange {
            start,
            end,
        }),
        map(line, |line| {
            // 8.4. Placement Shorthands: the grid-column, grid-row, and grid-area properties
            // When the second value is omitted, if the first value is a <custom-ident>, the grid-row-end/grid-column-end longhand is also set to that <custom-ident>; otherwise, it is set to auto.
            if let Line::Named(ident) = line {
                LineRange {
                    start: Line::from(ident),
                    end: Line::from(ident),
                }
            } else {
                LineRange {
                    start: line,
                    end: Line::Auto,
                }
            }
        }),
    ))(input)
}

impl<'a> LineRange<'a> {
    pub fn parse(input: &'a str) -> Result<Self, nom::error::Error<String>> {
        parse_standalone(input, line_range)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Area
////////////////////////////////////////////////////////////////////////////////////////////////////

fn area(input: &str) -> IResult<&str, Area> {
    map(
        tuple((
            line,
            opt(preceded(slash_sep, line)),
            opt(preceded(slash_sep, line)),
            opt(preceded(slash_sep, line)),
        )),
        |lines| {
            match lines {
                (row_start, Some(column_start), Some(row_end), Some(column_end)) => Area {
                    row: LineRange {
                        start: row_start,
                        end: row_end,
                    },
                    column: LineRange {
                        start: column_start,
                        end: column_end,
                    },
                },
                (row_start, Some(column_start), Some(row_end), None) => {
                    // 8.4. Placement Shorthands: the grid-column, grid-row, and grid-area properties
                    // When grid-column-end is omitted, if grid-column-start is a <custom-ident>, grid-column-end is set to that <custom-ident>; otherwise, it is set to auto.
                    Area {
                        row: LineRange {
                            start: row_start,
                            end: row_end,
                        },
                        column: LineRange {
                            start: column_start,
                            end: if let Line::Named(column_start_ident) = column_start {
                                Line::Named(column_start_ident)
                            } else {
                                Line::Auto
                            },
                        },
                    }
                }
                (row_start, Some(column_start), None, None) => {
                    // 8.4. Placement Shorthands: the grid-column, grid-row, and grid-area properties
                    // When grid-row-end is omitted, if grid-row-start is a <custom-ident>, grid-row-end is set to that <custom-ident>; otherwise, it is set to auto.
                    Area {
                        row: LineRange {
                            start: row_start,
                            end: if let Line::Named(row_start_ident) = row_start {
                                Line::Named(row_start_ident)
                            } else {
                                Line::Auto
                            },
                        },
                        column: LineRange {
                            start: column_start,
                            end: if let Line::Named(column_start_ident) = column_start {
                                Line::Named(column_start_ident)
                            } else {
                                Line::Auto
                            },
                        },
                    }
                }
                (row_start, None, None, None) => {
                    // 8.4. Placement Shorthands: the grid-column, grid-row, and grid-area properties
                    // When grid-column-start is omitted, if grid-row-start is a <custom-ident>, all four longhands are set to that value. Otherwise, it is set to auto.
                    let line = if let Line::Named(row_start_ident) = row_start {
                        Line::Named(row_start_ident)
                    } else {
                        Line::Auto
                    };

                    Area {
                        row: LineRange {
                            start: row_start,
                            end: line,
                        },
                        column: LineRange { start: line, end: line },
                    }
                }
                _ => unreachable!(),
            }
        },
    )(input)
}

impl<'a> Area<'a> {
    pub fn parse(input: &'a str) -> Result<Self, nom::error::Error<String>> {
        parse_standalone(input, area)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use crate::widget::grid::{
        Area, GridTemplate, Line,
        LineRange,
    };

    #[test]
    fn grid_specs() {
        let r = GridTemplate::parse("[start] 45px {45px} [end] / [name] 200px [type] 200px [value] 1fr").unwrap();
        eprintln!("{:?}", r);

        let r = GridTemplate::parse("45px {45px} / 200px 200px 1fr / 2dip 2dip").unwrap();
        eprintln!("{:?}", r);

        let r = GridTemplate::parse("40 20 / {55} / 5 10").unwrap();
        eprintln!("{:?}", r);
    }

    #[test]
    fn grid_spans() {
        // an area of the grid

        // row 0, col 0
        assert_eq!(
            Area::parse("0 / 0").unwrap(),
            Area {
                row: LineRange {
                    start: Line::Index(0),
                    end: Line::Auto
                },
                column: LineRange {
                    start: Line::Index(0),
                    end: Line::Auto
                },
            }
        );

        // row 0 OR col 0
        assert_eq!(
            LineRange::parse("0").unwrap(),
            LineRange {
                start: Line::Index(0),
                end: Line::Auto
            }
        );

        // from line 0 to 2
        assert_eq!(
            LineRange::parse("0 / 2").unwrap(),
            LineRange {
                start: Line::Index(0),
                end: Line::Index(2)
            }
        );

        // all columns of the implicit row after the last, given the following spec: "[last] / [col-start] 45px 200px 1fr [col-end]"
        assert_eq!(
            Area::parse("last / 0 / span 1 / -1").unwrap(),
            Area {
                row: LineRange {
                    start: Line::Named("last"),
                    end: Line::Span(1)
                },
                column: LineRange {
                    start: Line::Index(0),
                    end: Line::Index(-1)
                }
            }
        );
    }
}
