//! Parser for the grid definition/placement language.
use crate::widget::grid::{Area, GridTemplate, Line, LineRange, TrackBreadth};
use cssparser::Parser;
use kyute_common::{Length, UnitExt};
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

////////////////////////////////////////////////////////////////////////////////////////////////////
// TrackItem
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum TrackItem<'a> {
    LineTags(Vec<&'a str>),
    TrackSize(TrackSizePolicy),
    ImplicitTrackSize(TrackSizePolicy),
}

fn track_item(input: &str) -> IResult<&str, TrackItem> {
    context(
        "track_item",
        alt((
            map(line_tags, TrackItem::LineTags),
            map(length_or_auto, |size| TrackItem::TrackSize(TrackSizePolicy::new(size))),
            map(implicit_track, |size| {
                TrackItem::ImplicitTrackSize(TrackSizePolicy::new(size))
            }),
        )),
    )(input)
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// GridTemplate
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn grid_template(input: &str) -> IResult<&str, GridTemplate> {
    let (input, _) = space0(input)?;
    let (input, row_items) = separated_list1(space1, track_item)(input)?;
    let (input, _) = delimited(space0, char('/'), space0)(input)?;
    let (input, column_items) = separated_list1(space1, track_item)(input)?;
    let (input, _) = space0(input)?;
    let (input, gaps) = opt(preceded(
        delimited(space0, char('/'), space0),
        tuple((non_fractional_length, opt(preceded(space0, non_fractional_length)))),
    ))(input)?;

    let (row_gap, column_gap) = match gaps {
        None => (None, None),
        Some((gap_1, None)) => (Some(gap_1), Some(gap_1)),
        Some((gap_1, Some(gap_2))) => (Some(gap_1), Some(gap_2)),
    };

    let mut rows = Vec::new();
    let mut columns = Vec::new();
    let mut row_tags = Vec::new();
    let mut column_tags = Vec::new();
    let mut implicit_row_size = TrackSizePolicy::new(GridLength::Auto);
    let mut implicit_column_size = TrackSizePolicy::new(GridLength::Auto);

    for item in row_items {
        match item {
            TrackItem::LineTags(tags) => {
                for tag in tags {
                    row_tags.push((rows.len(), tag.to_string()))
                }
            }
            TrackItem::TrackSize(size) => {
                rows.push(size);
            }
            TrackItem::ImplicitTrackSize(size) => {
                implicit_row_size = size;
            }
        }
    }
    for item in column_items {
        match item {
            TrackItem::LineTags(tags) => {
                for tag in tags {
                    column_tags.push((columns.len(), tag.to_string()))
                }
            }
            TrackItem::TrackSize(size) => {
                columns.push(size);
            }
            TrackItem::ImplicitTrackSize(size) => {
                implicit_column_size = size;
            }
        }
    }

    let template = GridTemplate {
        rows,
        columns,
        row_tags,
        column_tags,
        implicit_row_size,
        implicit_column_size,
        row_gap,
        column_gap,
    };

    Ok((input, template))
}*/

impl GridTemplate {
    pub fn parse(input: &str) -> Result<Self, nom::error::Error<String>> {
        parse_standalone(input, grid_template)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Line
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn line(input: &str) -> IResult<&str, Line> {
    alt((
        map(tag("auto"), |_| Line::Auto),
        map(preceded(tag("span"), preceded(space0, integer_usize)), Line::Span),
        map(identifier, Line::Named),
        map(integer_i32, Line::Index),
    ))(input)
}*/

impl<'a> Line<'a> {
    pub fn parse(input: &'a str) -> Result<Self, nom::error::Error<String>> {
        parse_standalone(input, line)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// LineRange
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn slash_sep(input: &str) -> IResult<&str, char> {
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
}*/

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

////////////////////////////////////////////////////////////////////////////////////////////////////
// Area
////////////////////////////////////////////////////////////////////////////////////////////////////

/*fn area(input: &str) -> IResult<&str, Area> {
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
}*/

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
    use crate::widget::grid::{Area, GridTemplate, Line, LineRange};

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
