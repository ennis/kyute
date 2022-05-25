//! Parser for the grid definition/placement language.

use crate::widget::GridLength;
use kyute_common::{Length, UnitExt};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, digit1, space0, space1},
    combinator::{map, map_res, opt, peek, recognize},
    error::{context, make_error, ErrorKind, VerboseError},
    multi::{many0_count, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
};
use std::str::FromStr;

type ParseError = nom::Err<VerboseError<I>>;
pub type IResult<I, O> = Result<(I, O), ParseError>;

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

fn integer_u32(input: &str) -> IResult<&str, u32> {
    map_res(digit1, |s: &str| s.parse::<u32>())(input)
}

fn integer_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

fn unit(input: &str) -> IResult<&str, Unit> {
    // px, dip, etc.
    context(
        "unit",
        map(
            alt((tag("px"), tag("dip"), tag("in"), tag("pt"), tag("%"), tag("fr"))),
            |s: &str| match s {
                "px" => Unit::Px,
                "dip" => Unit::Dip,
                "%" => Unit::Percent,
                "pt" => Unit::Pt,
                "fr" => Unit::Fractional,
                _ => unreachable!(),
            },
        ),
    )(input)
}

/// Parses a length.
fn length(input: &str) -> IResult<&str, Length> {
    let (input, len) = integer_u32(input)?;
    let (input, unit) = opt(unit)(input)?;
    match unit {
        None => Ok((input, len.dip())),
        Some(Unit::Dip) => Ok((input, len.dip())),
        Some(Unit::Pt) => Ok((input, len.pt())),
        Some(Unit::In) => Ok((input, len.inch())),
        Some(Unit::Px) => Ok((input, len.px())),
        Some(Unit::Percent) => Ok((input, len.percent())),
        Some(Unit::Fractional) => Err(nom::Err::Error(make_error(input, ErrorKind::IsNot))),
    }
}

/// Parses a length.
fn grid_length(input: &str) -> IResult<&str, GridLength> {
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
    context(
        "length_or_auto",
        alt((map(tag("auto"), |_| GridLength::Auto), grid_length)),
    )(input)
}

fn identifier(input: &str) -> IResult<&str, &str> {
    context(
        "identifier",
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
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

/// Track template
fn track_template(input: &str) -> IResult<&str, GridLength> {
    context("track_template", delimited(char('{'), grid_length, char('}')))(input)
}

#[derive(Debug)]
enum TrackItem<'a> {
    LineTags(Vec<&'a str>),
    Track(GridLength),
    TrackTemplate(GridLength),
}

fn track_item(input: &str) -> IResult<&str, TrackItem> {
    context(
        "track_item",
        alt((
            map(line_tags, TrackItem::LineTags),
            map(length_or_auto, TrackItem::Track),
            map(track_template, TrackItem::TrackTemplate),
        )),
    )(input)
}

/// A template for a grid's rows, columns, and gaps.
#[derive(Debug)]
pub struct GridTemplate<'a> {
    rows: Vec<TrackItem<'a>>,
    columns: Vec<TrackItem<'a>>,
    row_gap: Option<Length>,
    column_gap: Option<Length>,
}

fn grid_spec(input: &str) -> IResult<&str, GridTemplate> {
    let (input, _) = space0(input)?;
    let (input, rows) = separated_list1(space1, track_item)(input)?;
    let (input, _) = delimited(space0, char('/'), space0)(input)?;
    let (input, columns) = separated_list1(space1, track_item)(input)?;
    let (input, _) = space0(input)?;
    let (input, gaps) = opt(preceded(
        delimited(space0, char('/'), space0),
        tuple((length, opt(length))),
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

/// Identifies a particular grid line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Line<'a> {
    /// Identifies a line by its name, as defined in the grid template.
    Named(&'a str),
    /// Identifies a line by its index, starting from the first line.
    Index(usize),
    /// Identifies a line by its index, starting from the *last* line.
    RevIndex(usize),
}

impl FromStr for Line<'a> {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

fn track_line(input: &str) -> IResult<&str, Line> {
    alt((
        map(identifier, Line::Named),
        map(integer_usize, Line::Index),
        map(preceded(char('$'), opt(preceded(char('-'), integer_usize))), |x| {
            Line::RevIndex(x.unwrap_or(0))
        }),
    ))
}

#[derive(Debug)]
enum LineSpan<'a> {
    SingleTrack(Line<'a>),
    Range { start_line: Line<'a>, end_line: Line<'a> },
    RangeTo(Line<'a>),
    RangeFrom(Line<'a>),
    Full,
}

fn track_span(input: &str) -> IResult<&str, LineSpan> {
    let (input, first_line) = opt(track_line)(input)?;
    let (input, _) = space0(input)?;
    let (input, dotdot) = opt(tag(".."))(input)?;
    let (input, second_line) = if let Some(dotdot) = dotdot {
        delimited(space0, opt(track_line), space0)(input)?
    } else {
        (input, None)
    };

    let range = match (first_line, dotdot, second_line) {
        // X
        (Some(first_line), None, None) => LineSpan::SingleTrack(first_line),
        // X..
        (Some(first_line), Some(_), None) => LineSpan::RangeFrom(first_line),
        // ..X
        (None, Some(_), Some(second_line)) => LineSpan::RangeTo(second_line),
        // X..X
        (Some(first_line), Some(_), Some(second_line)) => LineSpan::Range {
            start_line: first_line,
            end_line: second_line,
        },
        // ..
        (None, Some(_), None) => LineSpan::Full,
        // nothing?
        (None, None, None) => LineSpan::Full,
    };

    Ok((input, range))
}

fn grid_area(input: &str) -> IResult<&str, Area> {
    map(
        separated_pair(track_span, delimited(space0, char('/'), space0), track_span)(input)?,
        |(row_span, column_span)| Area { row_span, column_span },
    )
}

/// The parsed form of a grid area specifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Area<'a> {
    row_span: LineSpan<'a>,
    column_span: LineSpan<'a>,
}

impl FromStr for Area {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (rest, area) = grid_area(s.trim())?;
        if !rest.is_empty() {
            Err(ParseError::Error())
        }
        Ok(area)
    }
}

#[cfg(test)]
mod tests {
    use crate::widget::grid::parser::{grid_area, grid_spec, track_span, Area, Line, LineSpan};

    #[test]
    fn grid_specs() {
        let (_, r) = grid_spec("[start] 45px {45px} [end] / [name] 200px [type] 200px [value] 1fr").unwrap();
        eprintln!("{:?}", r);

        let (_, r) = grid_spec("45px {45px} / 200px 200px 1fr / 2dip 2dip").unwrap();
        eprintln!("{:?}", r);

        let (_, r) = grid_spec("40 20 / {55} / 5 10").unwrap();
        eprintln!("{:?}", r);
    }

    #[test]
    fn grid_spans() {
        // an area of the grid

        // row 0, col 0
        assert_eq!(
            grid_area("0 / 0").unwrap().1,
            Area {
                row_span: LineSpan::SingleTrack(Line::Index(0)),
                column_span: LineSpan::SingleTrack(Line::Index(0)),
            }
        );

        // row 0 OR col 0
        assert_eq!(track_span("0").unwrap(), LineSpan::SingleTrack(Line::Index(0)));
        // last line
        assert_eq!(track_span("$").unwrap(), LineSpan::SingleTrack(Line::RevIndex(0)));
        // from line 0 to 2
        assert_eq!(
            track_span("0 .. 2").unwrap(),
            LineSpan::Range {
                start_line: Line::Index(0),
                end_line: Line::Index(2)
            }
        );

        // all columns of the implicit row after the last, given the following spec: "{auto} [last] / [col-start] 45px 200px 1fr [col-end]"
        assert_eq!(
            grid_area("last / ..").unwrap(),
            Area {
                row_span: LineSpan::SingleTrack(Line::Named("last")),
                column_span: LineSpan::Full
            }
        );

        // same as above
        assert_eq!(
            grid_area("last / col-start .. col-end").unwrap(),
            Area {
                row_span: LineSpan::SingleTrack(Line::Named("last")),
                column_span: LineSpan::Range {
                    start_line: Line::Named("col-start"),
                    end_line: Line::Named("col-end")
                }
            }
        );

        // same as above
        assert_eq!(
            grid_area("$ / ..").unwrap(),
            Area {
                row_span: LineSpan::SingleTrack(Line::RevIndex(0)),
                column_span: LineSpan::Full
            }
        );

        assert_eq!(
            grid_area("..$-1 / ..").unwrap(),
            Area {
                row_span: LineSpan::RangeTo(Line::RevIndex(1)),
                column_span: LineSpan::Full
            }
        );
    }
}
