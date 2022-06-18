//! Grid layout property types.
use crate::{
    style::{values::length::length, StyleCtx, ToComputedValue},
    Length,
};
use cssparser::{ParseError, Parser, Token};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

/// Length of a grid track.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackBreadth {
    /// Size to content.
    Auto,
    /// Fixed size.
    Fixed(Length),
    /// Proportion of remaining space.
    Flex(f64),
}

impl Default for TrackBreadth {
    fn default() -> Self {
        TrackBreadth::Auto
    }
}

/// Length of a grid track after resolving non-flex lengths.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ComputedTrackBreadth {
    Auto,
    Fixed(f64),
    Flex(f64),
}

impl Default for ComputedTrackBreadth {
    fn default() -> Self {
        ComputedTrackBreadth::Auto
    }
}

impl ToComputedValue for TrackBreadth {
    type ComputedValue = ComputedTrackBreadth;

    fn to_computed_value(&self, context: &StyleCtx) -> ComputedTrackBreadth {
        match *self {
            TrackBreadth::Auto => ComputedTrackBreadth::Auto,
            TrackBreadth::Fixed(x) => x.to_computed_value(context),
            TrackBreadth::Flex(x) => ComputedTrackBreadth::Flex(x),
        }
    }
}

/// Sizing behavior of a grid track.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct TrackSize {
    min_size: TrackBreadth,
    max_size: TrackBreadth,
}

impl TrackSize {
    /// Defines a track that is sized according to the provided GridLength value.
    pub fn new(size: impl Into<TrackBreadth>) -> TrackSize {
        let size = size.into();
        TrackSize {
            min_size: size,
            max_size: size,
        }
    }

    /// Defines minimum and maximum sizes for the
    pub fn minmax(min_size: impl Into<TrackBreadth>, max_size: impl Into<TrackBreadth>) -> TrackSize {
        TrackSize {
            min_size: min_size.into(),
            max_size: max_size.into(),
        }
    }
}

impl From<TrackBreadth> for TrackSize {
    fn from(size: TrackBreadth) -> Self {
        TrackSize::new(size)
    }
}

/// Sizing behavior of a grid track.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ComputedTrackSize {
    pub min_size: ComputedTrackBreadth,
    pub max_size: ComputedTrackBreadth,
}

impl ToComputedValue for TrackSize {
    type ComputedValue = ComputedTrackSize;

    fn to_computed_value(&self, context: &StyleCtx) -> ComputedTrackSize {
        ComputedTrackSize {
            min_size: self.min_size.to_computed_value(context),
            max_size: self.max_size.to_computed_value(context),
        }
    }
}

/// List
#[derive(Clone, Debug)]
pub struct TrackList {
    sizes: Vec<TrackSize>,
    line_names: Vec<(usize, String)>,
}

#[derive(Clone, Debug, Default)]
pub struct ComputedTrackList {
    sizes: Vec<ComputedTrackSize>,
    line_names: Vec<(usize, String)>,
}

impl ToComputedValue for TrackList {
    type ComputedValue = ComputedTrackList;

    fn to_computed_value(&self, context: &StyleCtx) -> ComputedTrackList {
        ComputedTrackList {
            sizes: self.sizes.iter().map(|x| x.to_computed_value(context)).collect(),
            line_names: self.line_names.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Template {
    pub rows: TrackList,
    pub columns: TrackList,
}

#[derive(Clone, Debug, Default)]
pub struct ComputedTemplate {
    rows: ComputedTrackList,
    columns: ComputedTrackList,
}

impl ToComputedValue for Template {
    type ComputedValue = ComputedTemplate;

    fn to_computed_value(&self, context: &StyleCtx) -> ComputedTemplate {
        ComputedTemplate {
            rows: self.rows.to_computed_value(context),
            columns: self.rows.to_computed_value(context),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Line / LineRange
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Identifies a particular grid line or a line span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Line {
    Auto,
    /// Identifies a line by its name, as defined in the grid template.
    Named(String),
    /// Identifies a line by its index.
    Index(i32),
    Span(usize),
}

impl Default for Line {
    fn default() -> Self {
        Line::Auto
    }
}

impl From<i32> for Line {
    fn from(p: i32) -> Self {
        Line::Index(p)
    }
}

impl<'a> From<&'a str> for Line {
    fn from(s: &'a str) -> Self {
        Line::Named(s.to_owned())
    }
}

impl ToComputedValue for Line {
    type ComputedValue = Line;

    fn to_computed_value(&self, context: &StyleCtx) -> Line {
        self.clone()
    }
}

///////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LineRange {
    pub start: Line,
    pub end: Line,
}

impl From<Line> for LineRange {
    fn from(start: Line) -> Self {
        LineRange {
            start,
            end: Line::Span(1),
        }
    }
}

impl From<i32> for LineRange {
    fn from(p: i32) -> Self {
        LineRange {
            start: Line::Index(p),
            end: Line::Span(1),
        }
    }
}

impl From<usize> for LineRange {
    fn from(p: usize) -> Self {
        LineRange {
            start: Line::Index(p as i32),
            end: Line::Span(1),
        }
    }
}

impl From<Range<i32>> for LineRange {
    fn from(v: Range<i32>) -> Self {
        LineRange {
            start: Line::Index(v.start),
            end: Line::Index(v.end),
        }
    }
}

impl From<Range<usize>> for LineRange {
    fn from(v: Range<usize>) -> Self {
        LineRange {
            start: Line::Index(v.start as i32),
            end: Line::Index(v.end as i32),
        }
    }
}

impl From<RangeTo<i32>> for LineRange {
    fn from(v: RangeTo<i32>) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index(v.end),
        }
    }
}

impl From<RangeFrom<i32>> for LineRange {
    fn from(v: RangeFrom<i32>) -> Self {
        LineRange {
            start: Line::Index(v.start),
            end: Line::Index(-1),
        }
    }
}

impl From<RangeInclusive<i32>> for LineRange {
    fn from(v: RangeInclusive<i32>) -> Self {
        LineRange {
            start: Line::Index(*v.start()),
            end: Line::Index((*v.end() + 1) as i32),
        }
    }
}

impl From<RangeInclusive<usize>> for LineRange {
    fn from(v: RangeInclusive<usize>) -> Self {
        LineRange {
            start: Line::Index(*v.start() as i32),
            end: Line::Index((*v.end() + 1) as i32),
        }
    }
}

impl<'a> From<RangeToInclusive<i32>> for LineRange {
    fn from(v: RangeToInclusive<i32>) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index(v.end + 1),
        }
    }
}

impl From<RangeToInclusive<usize>> for LineRange {
    fn from(v: RangeToInclusive<usize>) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index((v.end + 1) as i32),
        }
    }
}

impl From<RangeFull> for LineRange {
    fn from(_: RangeFull) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index(-1),
        }
    }
}

/*
impl<'a> TryFrom<&'a str> for LineRange {
    type Error = nom::error::Error<String>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        LineRange::parse(input)
    }
}*/

/*impl<'a> From<&'a str> for TrackRange<'a> {
    fn from(v: &'a str) -> Self {
        GridSpan::Named(v)
    }
}*/

fn line_index(index: i32, line_count: usize) -> usize {
    if index < 0 {
        if (-index) as usize > line_count {
            warn!("track line overflow: {index}");
            0
        } else {
            (line_count as i32 + index) as usize
        }
    } else {
        index as usize
    }
}

/// The parsed form of a grid area specifier.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Area {
    pub row: LineRange,
    pub column: LineRange,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// parsers
////////////////////////////////////////////////////////////////////////////////////////////////////

impl TrackBreadth {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<TrackBreadth, ParseError<'i, ()>> {
        if let Ok(length) = input.try_parse(length)? {
            Ok(TrackBreadth::Fixed(length))
        } else {
            match input.next()? {
                Token::Ident(ident) if ident == "auto" => Ok(TrackBreadth::Auto),
                Token::Dimension { value, unit, .. } => match &**unit {
                    "fr" => Ok(TrackBreadth::Flex(value as f64)),
                    _ => Err(input.new_custom_error(())),
                },
                token => Err(input.new_unexpected_token_error(token.clone())),
            }
        }
    }
}

impl TrackSize {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<TrackSize, ParseError<'i, ()>> {
        let breadth = TrackBreadth::parse_impl(input)?;
        Ok(TrackSize {
            min_size: breadth,
            max_size: breadth,
        })
    }
}

pub(crate) fn grid_line_names<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<String>, ParseError<'i, ()>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let idents = input.parse_comma_separated(Parser::expect_ident)?;
        Ok(idents.iter().map(|x| x.to_string()).collect::<Vec<_>>())
    })
}

impl TrackList {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<TrackList, ParseError<'i, ()>> {
        let mut line_names: Vec<(usize, String)> = vec![];
        let mut sizes = vec![];
        loop {
            if let Ok(names) = input.try_parse(grid_line_names) {
                let i = sizes.len();
                for name in names {
                    line_names.push((i, name));
                }
            }

            if let Ok(track_size) = input.try_parse(TrackSize::parse_impl)? {
                sizes.push(track_size);
            } else {
                break;
            }
        }

        input.expect_exhausted()?;
        Ok(TrackList { sizes, line_names })
    }
}

impl Line {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Line, ParseError<'i, ()>> {
        let first = input.next()?;
        let second = input.try_parse(|input| input.next());
        match (first, second) {
            // auto
            (Ok(Token::Ident(id)), Err(_)) if &**id == "auto" => Ok(Line::Auto),
            // span N
            (
                Ok(Token::Ident(id)),
                Ok(Token::Number {
                    int_value: Some(span), ..
                }),
            ) if &**id == "span" => {
                // FIXME check for negative values
                Ok(Line::Span(*span as usize))
            }
            // N span
            (
                Ok(Token::Number {
                    int_value: Some(span), ..
                }),
                Ok(Token::Ident(id)),
            ) if &**id == "span" => {
                // FIXME check for negative values
                Ok(Line::Span(*span as usize))
            }
            // integer
            (
                Ok(Token::Number {
                    int_value: Some(line_index),
                    ..
                }),
                Err(_),
            ) => Ok(Line::Index(*line_index)),
            // <custom-ident>
            (Ok(Token::Ident(id)), Err(_)) => Ok(Line::Named(id.to_string())),
            _ => Err(input.new_custom_error(())),
        }
    }
}

impl LineRange {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<LineRange, ParseError<'i, ()>> {
        // FIXME this is definitely not what the spec says
        let start = Line::parse_impl(input)?;
        if let Ok(_) = input.try_parse(|input| input.expect_delim('/')) {
            let end = Line::parse_impl(input)?;
            Ok(LineRange { start, end })
        } else {
            Ok(LineRange {
                start: start.clone(),
                end: Line::Auto,
            })
        }
    }
}

impl Area {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<Area, ParseError<'i, ()>> {
        // FIXME this is definitely not what the spec says
        let row_start = Line::parse_impl(input)?;
        let column_start = input.try_parse(Line::parse_impl);
        let row_end = input.try_parse(Line::parse_impl);
        let column_end = input.try_parse(Line::parse_impl);
        Ok(Area {
            row: LineRange {
                start: row_start,
                end: row_end.unwrap_or_default(),
            },
            column: LineRange {
                start: column_start.unwrap_or_default(),
                end: column_end.unwrap_or_default(),
            },
        })
    }
}
