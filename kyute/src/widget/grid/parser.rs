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
    sequence::{delimited, pair, preceded, tuple},
};

pub type IResult<I, O> = Result<(I, O), nom::Err<VerboseError<I>>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Unit {
    Px,
    Pt,
    In,
    Dip,
    Percent,
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

#[derive(Debug)]
struct GridSpec<'a> {
    rows: Vec<TrackItem<'a>>,
    columns: Vec<TrackItem<'a>>,
    row_gap: Option<Length>,
    column_gap: Option<Length>,
}

fn grid_spec(input: &str) -> IResult<&str, GridSpec> {
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
        None => GridSpec {
            rows,
            columns,
            row_gap: None,
            column_gap: None,
        },
        Some((gap_1, None)) => GridSpec {
            rows,
            columns,
            row_gap: Some(gap_1),
            column_gap: Some(gap_1),
        },
        Some((gap_1, Some(gap_2))) => GridSpec {
            rows,
            columns,
            row_gap: Some(gap_1),
            column_gap: Some(gap_2),
        },
    };
    Ok((input, spec))
}

#[derive(Debug)]
enum GridTrackLine<'a> {
    Named(&'a str),
    Index(usize),
    RevIndex(usize),
}

fn track_line(input: &str) -> IResult<&str, GridTrackLine> {
    alt((
        map(identifier, GridTrackLine::Named),
        map(integer_usize, GridTrackLine::Index),
        map(
            preceded(char('$'), opt(preceded(char('-'), integer_usize))),
            GridTrackLine::RevIndex,
        ),
    ))
}

#[derive(Debug)]
enum GridTrackRange<'a> {
    SingleTrack(GridTrackLine<'a>),
    Range {
        start_line: GridTrackLine<'a>,
        end_line: GridTrackLine<'a>,
    },
    RangeTo(GridTrackLine<'a>),
    RangeFrom(GridTrackLine<'a>),
    Full,
}

fn track_span(input: &str) -> IResult<&str, GridTrackRange> {
    let (input, first_line) = opt(track_line)(input)?;
    let (input, _) = space0(input)?;
    let (input, dotdot) = opt(tag(".."))(input)?;
    let (input, second_line) = if let Some(dotdot) = dotdot {
        delimited(space0, opt(track_line), space0)(input)?
    } else {
        (input, None)
    };

    let range = match (first_line, dotdot, second_line) {
        (Some(first_line), None, None) => GridTrackRange::SingleTrack(first_line),
        (Some(first_line), Some(_), None) => GridTrackRange::RangeFrom(first_line),
        (None, Some(_), Some(second_line)) => GridTrackRange::RangeTo(second_line),
        (Some(first_line), Some(_), Some(second_line)) => GridTrackRange::Range {
            start_line: first_line,
            end_line: second_line,
        },
        (None, Some(_), None) => GridTrackRange::Full,
        (None, None, None) => GridTrackRange::Full,
    };

    Ok((input, range))
}

#[derive(Debug)]
struct GridArea {}

fn grid_area(input: &str) -> IResult<&str, GridArea> {}

#[cfg(test)]
mod tests {
    use crate::widget::grid::parser::grid_spec;

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

        "0 / 0"; // row 0, col 0
        "0"; // row 0 OR col 0
        "0 .. 2"; // rows 0..2 (0,1) OR cols 0..2 (0,1)

        "last / .."; // all columns of the implicit row after the last, given the following spec: "{auto} [last] / [col-start] 45px 200px 1fr [col-end]"
        "last / col-start .. col-end"; // same as above
        "$ / .."; // same as above
        "$-1.. / .."; // all columns of
    }
}
