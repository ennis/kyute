use crate::{
    bloom::Bloom,
    cache,
    core::DebugNode,
    css::parse_from_str,
    drawing,
    drawing::{Paint, PaintCtxExt, Shape, ToSkia},
    style,
    widget::prelude::*,
    Color, Data, EnvKey, Length, RoundToPixel, State, WidgetId,
};
use cssparser::{ParseError, Parser, Token};
use kyute::css::parse_css_length;
use lazy_static::lazy_static;
use std::{
    cell::Cell,
    convert::{TryFrom, TryInto},
    mem,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    sync::Arc,
};

pub const SHOW_GRID_LAYOUT_LINES: EnvKey<bool> = builtin_env_key!("kyute.grid.show-layout-lines");

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

impl From<Length> for TrackBreadth {
    fn from(length: Length) -> Self {
        TrackBreadth::Fixed(length)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Data)]
pub enum JustifyItems {
    Start,
    End,
    Center,
    // TODO currently ignored
    Stretch,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Data)]
pub enum AlignItems {
    Start,
    End,
    Center,
    // TODO currently ignored
    Stretch,
    Baseline,
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

impl TrackBreadth {
    pub(crate) fn parse_impl<'i>(input: &mut Parser<'i, '_>) -> Result<TrackBreadth, ParseError<'i, ()>> {
        if let Ok(length) = input.try_parse(parse_css_length) {
            Ok(TrackBreadth::Fixed(length))
        } else {
            match input.next()? {
                Token::Ident(ident) if &**ident == "auto" => Ok(TrackBreadth::Auto),
                Token::Dimension { value, unit, .. } => match &**unit {
                    "fr" => Ok(TrackBreadth::Flex(*value as f64)),
                    _ => Err(input.new_custom_error(())),
                },
                token => {
                    let token = token.clone();
                    Err(input.new_unexpected_token_error(token))
                }
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

/// Orientation of a grid track.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Axis {
    /// Row (or _inline_) axis (follows the text).
    Row,
    /// Column (or _block_) axis, perpendicular to the flow of text.
    Column,
}

/*/// Returns the size of a box along the specified axis.
fn size_along(axis: Axis, size: Size) -> f64 {
    // TODO depends on the writing mode
    match axis {
        Axis::Row => size.width,
        Axis::Column => size.height,
    }
}*/

/// Returns the size of a box along the specified axis.
fn size_across(axis: Axis, size: Size) -> f64 {
    // TODO depends on the writing mode
    match axis {
        Axis::Row => size.height,
        Axis::Column => size.width,
    }
}

/// List of tracks.
#[derive(Clone, Debug, Default)]
pub struct TrackList {
    pub sizes: Vec<TrackSize>,
    pub line_names: Vec<(usize, String)>,
}

fn grid_line_names<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<String>, ParseError<'i, ()>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let idents = input.parse_comma_separated(|input| Ok(input.expect_ident()?.clone()))?;
        Ok(idents.iter().map(|x| x.to_string()).collect::<Vec<_>>())
    })
}

impl TrackList {
    pub(crate) fn parse_css<'i>(input: &mut Parser<'i, '_>) -> Result<TrackList, ParseError<'i, ()>> {
        let mut line_names: Vec<(usize, String)> = vec![];
        let mut sizes = vec![];
        loop {
            if let Ok(names) = input.try_parse(grid_line_names) {
                let i = sizes.len();
                for name in names {
                    line_names.push((i, name));
                }
            }

            if let Ok(track_size) = input.try_parse(TrackSize::parse_impl) {
                sizes.push(track_size);
            } else {
                break;
            }
        }

        Ok(TrackList { sizes, line_names })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// GridTemplate
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A template for a grid's rows, columns.
#[derive(Default, Debug)]
pub struct GridTemplate {
    pub rows: TrackList,
    pub columns: TrackList,
}

impl GridTemplate {
    pub fn new() -> GridTemplate {
        GridTemplate::default()
    }

    /*pub fn push_row(&mut self, size: impl Into<TrackSizePolicy>) {
        self.rows.push(size.into());
    }

    pub fn push_row_tag(&mut self, tag: impl Into<String>) {
        self.row_tags.push((self.rows.len(), tag.into()));
    }

    pub fn push_column(&mut self, size: impl Into<TrackSizePolicy>) {
        self.columns.push(size.into());
    }

    pub fn push_column_tag(&mut self, tag: impl Into<String>) {
        self.column_tags.push((self.columns.len(), tag.into()));
    }*/
}

impl GridTemplate {
    pub(crate) fn parse_css<'i>(input: &mut Parser<'i, '_>) -> Result<GridTemplate, ParseError<'i, ()>> {
        // TODO
        let rows = TrackList::parse_css(input)?;
        input.expect_delim('/')?;
        let columns = TrackList::parse_css(input)?;
        Ok(GridTemplate { rows, columns })
    }
}

impl<'a> TryFrom<&'a str> for GridTemplate {
    type Error = ParseError<'a, ()>;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match parse_from_str(value, GridTemplate::parse_css) {
            Ok(val) => Ok(val),
            Err(err) => {
                warn!("GridTemplate parse error: {:?}", err);
                Err(err)
            }
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
        Line::Named(s.to_string())
    }
}

impl Line {
    /// Parses a <grid-line> CSS value.
    pub(crate) fn parse_css<'a, 'b>(input: &mut Parser<'a, 'b>) -> Result<Line, ParseError<'a, ()>> {
        let first = input.next()?.clone();
        let second = input.try_parse(|input| input.next().cloned());
        match (first, second) {
            // auto
            (Token::Ident(id), Err(_)) if &*id == "auto" => Ok(Line::Auto),
            // span N
            (
                Token::Ident(id),
                Ok(Token::Number {
                    int_value: Some(span), ..
                }),
            ) if &*id == "span" => {
                // FIXME check for negative values
                Ok(Line::Span(span as usize))
            }
            // N span
            (
                Token::Number {
                    int_value: Some(span), ..
                },
                Ok(Token::Ident(id)),
            ) if &*id == "span" => {
                // FIXME check for negative values
                Ok(Line::Span(span as usize))
            }
            // integer
            (
                Token::Number {
                    int_value: Some(line_index),
                    ..
                },
                Err(_),
            ) => Ok(Line::Index(line_index)),
            // <custom-ident>
            (Token::Ident(id), Err(_)) => Ok(Line::Named(id.to_string())),
            _ => Err(input.new_custom_error(())),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  LineRange
////////////////////////////////////////////////////////////////////////////////////////////////////

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

impl From<RangeToInclusive<i32>> for LineRange {
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

impl LineRange {
    /// Parses the value of a `grid-row` or `grid-column` property declaration.
    pub(crate) fn parse_impl<'a>(input: &mut Parser<'a, '_>) -> Result<LineRange, ParseError<'a, ()>> {
        // FIXME this is definitely not what the spec says
        let start = Line::parse_css(input)?;
        if let Ok(_) = input.try_parse(|input| input.expect_delim('/')) {
            let end = Line::parse_css(input)?;
            Ok(LineRange { start, end })
        } else {
            Ok(LineRange {
                start: start.clone(),
                end: Line::Auto,
            })
        }
    }
}

impl<'a> TryFrom<&'a str> for LineRange {
    type Error = ParseError<'a, ()>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        parse_from_str(input, LineRange::parse_impl)
    }
}

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

impl LineRange {
    fn resolve(&self, named_lines: &[(usize, String)], line_count: usize) -> (Option<usize>, usize) {
        if let (Line::Span(_), Line::Span(_)) = (&self.start, &self.end) {
            warn!("invalid line range");
            return (None, 1);
        }

        let mut start = None;
        let mut end = None;
        let mut span = None;

        match self.start {
            Line::Auto => {
                //if let Line::
            }
            Line::Named(ref ident) => {
                start = named_lines
                    .iter()
                    .find_map(|(line, name)| if name == ident { Some(line) } else { None })
                    .cloned();
            }
            Line::Index(index) => {
                start = Some(line_index(index, line_count));
            }
            Line::Span(s) => {
                span = Some(s);
            }
        }

        match self.end {
            Line::Auto => {
                //if let Line::
            }
            Line::Named(ref ident) => {
                end = named_lines
                    .iter()
                    .find_map(|(line, name)| if name == ident { Some(line) } else { None })
                    .cloned();
            }
            Line::Index(index) => {
                end = Some(line_index(index, line_count));
            }
            Line::Span(s) => {
                if span.is_some() {
                    warn!("invalid span");
                } else {
                    span = Some(s);
                }
            }
        }

        match (start, span, end) {
            // X / Y
            (Some(start), None, Some(end)) => (Some(start), end - start),
            // X / span Y
            (Some(start), Some(span), None) => (Some(start), span),
            // X / auto
            (Some(start), None, None) => (Some(start), 1),
            // span X / Y
            (None, Some(span), Some(end)) => (Some(end - span), span),
            // auto / end
            (None, None, Some(end)) => (Some(end - 1), 1),
            // span X
            (None, Some(span), None) => (None, span),
            (None, None, None) => (None, 1),
            _ => unreachable!(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Grid area
////////////////////////////////////////////////////////////////////////////////////////////////////

/// The parsed form of a grid area specifier.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct Area {
    row: LineRange,
    column: LineRange,
}

/*impl<'a> TryFrom<&'a str> for Area<'a> {
    type Error = nom::error::Error<String>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        Area::parse(input)
    }
}*/

impl<Rows, Columns> From<(Rows, Columns)> for Area
where
    Rows: Into<LineRange>,
    Columns: Into<LineRange>,
{
    fn from((rows, columns): (Rows, Columns)) -> Self {
        Area {
            row: rows.into(),
            column: columns.into(),
        }
    }
}

impl Area {
    /// Parses the value of a `grid-area` CSS property.
    pub(crate) fn parse_impl<'a>(input: &mut Parser<'a, '_>) -> Result<Area, ParseError<'a, ()>> {
        // FIXME this is definitely not what the spec says
        let row_start = Line::parse_css(input)?;
        let column_start = input.try_parse(Line::parse_css);
        let row_end = input.try_parse(Line::parse_css);
        let column_end = input.try_parse(Line::parse_css);
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

#[derive(Copy, Clone, Debug)]
struct DefiniteArea {
    /// If None, use flow
    row: Option<usize>,
    column: Option<usize>,
    row_span: usize,
    column_span: usize,
}

impl DefiniteArea {
    fn is_null(&self) -> bool {
        self.row_span == 0 || self.column_span == 0
    }
}

impl Area {
    fn resolve(&self, grid: &Grid) -> DefiniteArea {
        let (row, row_span) = self
            .row
            .resolve(&grid.template.rows.line_names, grid.template_row_count() + 1);
        let (column, column_span) = self
            .column
            .resolve(&grid.template.columns.line_names, grid.template_column_count() + 1);

        DefiniteArea {
            row,
            column,
            row_span,
            column_span,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// FlowCursor
////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////
// Insertable
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Items that can be inserted into a grid.
///
/// Either widgets or a grid placement wrapper around a widget (see `Placement`).
pub trait Insertable {
    fn insert(self, grid: &mut Grid);
}

impl<W> Insertable for W
where
    W: Widget + Sized + 'static,
{
    fn insert(self, grid: &mut Grid) {
        grid.place(Area::default(), 0, Arc::new(WidgetPod::new(self)));
    }
}

macro_rules! tuple_insertable {
    () => {};
    ( $w:ident : $t:ident, $($ws:ident : $ts:ident, )* ) => {
        impl<$t, $($ts,)*> Insertable for ($t, $($ts,)* ) where
            $t: Insertable + 'static,
            $( $ts: Insertable + 'static ),*
        {
            fn insert(self, grid: &mut Grid)
            {
                let ($w, $($ws,)*) = self;
                $w.insert(grid);
                $($ws.insert(grid);)*
            }
        }

        tuple_insertable!{$($ws : $ts,)*}
    };
}

tuple_insertable! {
    w1: T1,
    w2: T2,
    w3: T3,
    w4: T4,
    w5: T5,
    w6: T6,
    w7: T7,
    w8: T8,
    w9: T9,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// GridPlacer
////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Placement<W> {
    area: Area,
    widget: W,
}

impl<W> Placement<W> {
    pub fn new(widget: W) -> Placement<W> {
        Placement {
            area: Default::default(),
            widget,
        }
    }

    pub fn grid_row_start(mut self, line: impl TryInto<Line>) -> Self {
        self.area.row.start = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_row_end(mut self, line: impl TryInto<Line>) -> Self {
        self.area.row.end = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_column_start(mut self, line: impl TryInto<Line>) -> Self {
        self.area.column.start = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_column_end(mut self, line: impl TryInto<Line>) -> Self {
        self.area.column.end = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_row_span(mut self, len: usize) -> Self {
        self.area.row.end = Line::Span(len);
        self
    }

    pub fn grid_column_span(mut self, len: usize) -> Self {
        self.area.column.end = Line::Span(len);
        self
    }

    pub fn grid_row(mut self, range: impl TryInto<LineRange>) -> Self {
        self.area.row = range.try_into().unwrap_or_default();
        self
    }

    pub fn grid_column(mut self, range: impl TryInto<LineRange>) -> Self {
        self.area.column = range.try_into().unwrap_or_default();
        self
    }

    pub fn grid_area(mut self, area: impl TryInto<Area>) -> Self {
        self.area = area.try_into().unwrap_or_default();
        self
    }
}

impl<W> Insertable for Placement<W>
where
    W: Widget + 'static,
{
    fn insert(self, grid: &mut Grid) {
        grid.place(self.area, 1, Arc::new(WidgetPod::new(self.widget)));
    }
}

pub trait GridLayoutExt: Widget + Sized {
    fn grid_row_start<'a>(self, line: impl TryInto<Line>) -> Placement<Self> {
        Placement::new(self).grid_row_start(line)
    }

    fn grid_row_end<'a>(self, line: impl TryInto<Line>) -> Placement<Self> {
        Placement::new(self).grid_row_start(line)
    }

    fn grid_column_start<'a>(self, line: impl TryInto<Line>) -> Placement<Self> {
        Placement::new(self).grid_column_start(line)
    }

    fn grid_column_end<'a>(self, line: impl TryInto<Line>) -> Placement<Self> {
        Placement::new(self).grid_column_end(line)
    }

    fn grid_row_span<'a>(self, len: usize) -> Placement<Self> {
        Placement::new(self).grid_row_span(len)
    }

    fn grid_column_span<'a>(self, len: usize) -> Placement<Self> {
        Placement::new(self).grid_column_span(len)
    }

    fn grid_row<'a>(self, range: impl TryInto<LineRange>) -> Placement<Self> {
        Placement::new(self).grid_row(range)
    }

    fn grid_column<'a>(self, range: impl TryInto<LineRange>) -> Placement<Self> {
        Placement::new(self).grid_column(range)
    }

    fn grid_area<'a>(self, area: impl TryInto<Area>) -> Placement<Self> {
        Placement::new(self).grid_area(area)
    }
}

impl<W> GridLayoutExt for W where W: Widget + Sized {}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Grid
////////////////////////////////////////////////////////////////////////////////////////////////////

/// An item placed within a grid.
#[derive(Clone, Debug)]
struct GridItem {
    /// Specified area.
    area: DefiniteArea,
    row_range: Cell<(usize, usize)>,
    column_range: Cell<(usize, usize)>,
    z_order: i32,
    widget: Arc<WidgetPod>,
}

impl GridItem {
    fn row_range(&self) -> Range<usize> {
        let (start, end) = self.row_range.get();
        start..end
    }

    fn column_range(&self) -> Range<usize> {
        let (start, end) = self.column_range.get();
        start..end
    }

    fn is_in_track(&self, axis: Axis, index: usize) -> bool {
        // "grid line" items (those with row_range.len() == 0 or column_range.len() == 0)
        // are not considered to belong to any track, and don't intervene during track sizing
        if self.row_range().is_empty() || self.column_range().is_empty() {
            return false;
        }
        match axis {
            Axis::Row => self.row_range().start == index,
            Axis::Column => self.column_range().start == index,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridTrackLayout {
    pub pos: f64,
    pub size: f64,
}

struct ComputeTrackSizeResult {
    layout: Vec<GridTrackLayout>,
    size: f64,
}

#[derive(Clone, Debug, Default)]
struct Computed {
    row_layout: Vec<GridTrackLayout>,
    column_layout: Vec<GridTrackLayout>,
    width: f64,
    height: f64,
    show_grid_lines: bool,
    style: ComputedStyle,
}

#[derive(Clone, Debug, Default)]
struct ComputedStyle {
    row_gap: f64,
    column_gap: f64,
    row_background: drawing::Paint,
    alternate_row_background: drawing::Paint,
    row_gap_background: drawing::Paint,
    column_gap_background: drawing::Paint,
}

/*impl Data for Computed {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}*/

/// Visual style of a grid layout container.
#[derive(Clone, Debug, Default)]
pub struct GridStyle {
    pub row_gap: Length,
    pub column_gap: Length,
    //pub align_items: AlignItems,
    //pub justify_items: JustifyItems,
    /// Row background.
    pub row_background: style::Image,
    /// Alternate row background.
    pub alternate_row_background: style::Image,
    /// Row gap background.
    pub row_gap_background: style::Image,
    /// Column gap background.
    pub column_gap_background: style::Image,
}

/// Grid layout container.
///
/// TODO it's a bit heavy for laying out two buttons in a column...
#[derive(Debug)]
pub struct Grid {
    id: WidgetId,
    /// Visual style.
    style: Arc<GridStyle>,
    /// Grid row/column templates.
    template: Arc<GridTemplate>,
    /// List of grid items: widgets to be positioned inside the grid.
    items: Vec<GridItem>,
    /// Row template.
    implicit_row_size: TrackBreadth,
    implicit_column_size: TrackBreadth,
    auto_flow_dir: FlowDirection,
    align_items: AlignItems,
    justify_items: JustifyItems,
    /// Computed layout & style values.
    computed: State<Arc<Computed>>,
    cached_child_filter: Cell<Option<Bloom<WidgetId>>>,
}

/// Returns the size of a column span
fn track_span_width(layout: &[GridTrackLayout], span: Range<usize>, gap: f64) -> f64 {
    layout[span.clone()].iter().map(|x| x.size).sum::<f64>() + gap * (span.len() as isize - 1).max(0) as f64
}

lazy_static! {
    //static ref DEFAULT_GRID_STYLE: Arc<GridStyle> = Arc::new(GridStyle::default());
    static ref DEFAULT_GRID_TEMPLATE: Arc<GridTemplate> = Arc::new(GridTemplate::default());
}

impl Grid {
    /// Creates a new grid with the specified template.
    pub fn new(template: Arc<GridTemplate>) -> Grid {
        Grid {
            id: WidgetId::here(),
            template,
            items: vec![],
            implicit_row_size: TrackBreadth::Auto,
            implicit_column_size: TrackBreadth::Auto,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            auto_flow_dir: FlowDirection::Row,
            style: Arc::new(GridStyle::default()),
            computed: cache::state(|| Default::default()),
            cached_child_filter: Cell::new(None),
        }
    }

    pub fn with_template(template: impl TryInto<GridTemplate>) -> Grid {
        Grid::new(Arc::new(template.try_into().unwrap_or_else(|_err| {
            warn!("invalid grid template");
            GridTemplate::default()
        })))
    }

    /// Creates a single-column grid.
    pub fn column(width: impl Into<TrackBreadth>) -> Grid {
        let mut template = GridTemplate::new();
        template.columns.sizes.push(TrackSize::new(width));
        Grid::new(Arc::new(template))
    }

    /// Creates a single-row grid.
    pub fn row(height: impl Into<TrackBreadth>) -> Grid {
        let mut template = GridTemplate::new();
        template.rows.sizes.push(TrackSize::new(height));
        let mut grid = Grid::new(Arc::new(template));
        grid.auto_flow_dir = FlowDirection::Column;
        grid
    }

    /// Returns the number of columns in the grid template.
    pub fn template_column_count(&self) -> usize {
        self.template.columns.sizes.len()
    }

    /// Returns the number of rows in the grid template.
    pub fn template_row_count(&self) -> usize {
        self.template.rows.sizes.len()
    }

    /// Inserts items into the grid.
    pub fn insert(&mut self, items: impl Insertable) {
        items.insert(self);
    }

    /// Place an item at the specified location into the grid.
    ///
    /// Does not affect the current insertion cursor.
    pub fn place(&mut self, area: impl Into<Area>, z_order: i32, widget: Arc<WidgetPod>) {
        let mut area = area.into().resolve(self);
        if area.is_null() {
            warn!(
                "null grid area specified, widget {:?}({}) will not be inserted in the grid",
                widget.inner().widget_id(),
                widget.inner().debug_name(),
            );
            return;
        }
        self.items.push(GridItem {
            area,
            column_range: Cell::new((0, 0)),
            row_range: Cell::new((0, 0)),
            widget,
            z_order,
        });
    }

    pub fn set_implicit_row_size(&mut self, height: impl Into<TrackBreadth>) {
        self.implicit_row_size = height.into();
    }

    pub fn set_implicit_column_size(&mut self, width: impl Into<TrackBreadth>) {
        self.implicit_column_size = width.into();
    }

    /// Sets the auto flow direction
    pub fn set_auto_flow(&mut self, flow_direction: FlowDirection) {
        self.auto_flow_dir = flow_direction;
    }

    /*/// Returns the grid layout computed during layout.
    ///
    /// Returns none if not calculated yet (called before layout).
    pub fn get_layout(&self) -> Option<&CachedGridLayout> {}*/

    // Returns the current number of rows
    //pub fn row_count(&self) -> usize {
    //    self.row_definitions.len()
    //}

    // Returns the current number of columns
    // FIXME this should return the number of columns in the template
    //pub fn column_count(&self) -> usize {
    //   self.column_definitions.len()
    //}

    /// Sets the size of the gap between rows.
    pub fn set_row_gap(&mut self, gap: impl Into<Length>) {
        Arc::make_mut(&mut self.style).row_gap = gap.into();
    }

    /// Sets the size of the gap between columns.
    pub fn set_column_gap(&mut self, gap: impl Into<Length>) {
        Arc::make_mut(&mut self.style).column_gap = gap.into();
    }

    pub fn set_align_items(&mut self, align_items: AlignItems) {
        self.align_items = align_items;
    }

    pub fn set_justify_items(&mut self, justify_items: JustifyItems) {
        self.justify_items = justify_items;
    }

    pub fn set_row_background(&mut self, row_background: impl Into<style::Image>) {
        Arc::make_mut(&mut self.style).row_background = row_background.into();
    }

    pub fn set_alternate_row_background(&mut self, alternate_row_background: impl Into<style::Image>) {
        Arc::make_mut(&mut self.style).alternate_row_background = alternate_row_background.into();
    }

    pub fn set_row_gap_background(&mut self, bg: impl Into<style::Image>) {
        Arc::make_mut(&mut self.style).row_gap_background = bg.into();
    }

    pub fn set_column_gap_background(&mut self, bg: impl Into<style::Image>) {
        Arc::make_mut(&mut self.style).column_gap_background = bg.into();
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum FlowDirection {
    /// Fill rows first
    Row,
    /// Fill colums first
    Column,
}

/// Helper to place items within a grid.
#[derive(Debug)]
struct FlowCursor {
    row: usize,
    column: usize,
    row_len: usize,
    flow: FlowDirection,
}

impl FlowCursor {
    /// Advances the cursor to the specified column, possibly going to the next row if necessary.
    fn align(&mut self, column: usize) {
        if self.column < column {
            self.column = column;
        } else if self.column > column {
            self.row += 1;
            self.column = column;
        }
    }

    /// Advances the cursor by the specified row/column span.
    fn next(&mut self, row_span: usize, column_span: usize) -> (usize, usize) {
        let (row, column) = (self.row, self.column);
        self.column += column_span;
        if self.column >= self.row_len {
            self.row += row_span;
            self.column = 0;
        }
        (row, column)
    }

    /*fn place_helper(
        &mut self,
        row: usize,
        row_span: usize,
        column: usize,
        column_span: usize,
    ) -> (Range<usize>, Range<usize>) {
        match self.flow {
            FlowDirection::Row => (row..(row + row_span), column..(column + column_span)),
            FlowDirection::Column => (column..(column + column_span), row..(row + row_span)),
        }
    }*/

    fn place(&mut self, area: DefiniteArea) -> (Range<usize>, Range<usize>) {
        let mut row = area.row;
        let mut column = area.column;
        let mut row_span = area.row_span;
        let mut column_span = area.column_span;

        if self.flow == FlowDirection::Column {
            mem::swap(&mut row, &mut column);
            mem::swap(&mut row_span, &mut column_span);
        }

        let mut rows;
        let mut columns;

        match (row, column) {
            (Some(row), Some(column)) => {
                rows = row..(row + row_span);
                columns = column..(column + column_span);
            }
            (Some(row), None) => {
                // TODO packing
                rows = row..(row + row_span);
                columns = 0..column_span;
            }
            (None, col) => {
                if let Some(column) = col {
                    self.align(column);
                }
                let (row, column) = self.next(row_span, column_span);
                rows = row..(row + row_span);
                columns = column..(column + column_span);
            }
        }

        if self.flow == FlowDirection::Column {
            mem::swap(&mut rows, &mut columns);
        }

        (rows, columns)
    }
}

impl Grid {
    /// Position items inside the grid.
    fn position_items(&self) -> (usize, usize) {
        trace!(
            "=== [{:?}] positioning {} items ===",
            self.widget_id(),
            self.items.len()
        );
        trace!(
            "{} template rows, {} template columns, autoflow: {:?}",
            self.template.rows.sizes.len(),
            self.template.columns.sizes.len(),
            self.auto_flow_dir
        );

        let mut final_row_count = self.template.rows.sizes.len();
        let mut final_column_count = self.template.columns.sizes.len();

        let mut flow_cursor = FlowCursor {
            row: 0,
            column: 0,
            row_len: match self.auto_flow_dir {
                FlowDirection::Row => self.template.columns.sizes.len(),
                FlowDirection::Column => self.template.rows.sizes.len(),
            },
            flow: self.auto_flow_dir,
        };

        for item in self.items.iter() {
            if item.area.is_null() {
                // this should not happen because we check for null areas when adding the item to
                // the grid, but check it here as well for good measure
                error!("null grid area during placement (id={:?})", item.widget.widget_id());
                continue;
            }

            let (row_range, column_range) = flow_cursor.place(item.area);
            final_row_count = final_row_count.max(row_range.end);
            final_column_count = final_column_count.max(column_range.end);

            trace!(
                "{:?}: rows {}..{} columns {}..{} (area = {:?}, cursor = {:?})",
                item.widget.widget_id(),
                row_range.start,
                row_range.end,
                column_range.start,
                column_range.end,
                item.area,
                flow_cursor
            );

            item.row_range.set((row_range.start, row_range.end));
            item.column_range.set((column_range.start, column_range.end));
        }

        trace!(
            "final track count: rows={} columns={}",
            final_row_count,
            final_column_count
        );

        (final_row_count, final_column_count)
    }

    /// Computes the sizes of rows or columns.
    ///
    /// * `available_space`: max size across track direction (columns => max width, rows => max height).
    /// * `column_sizes`: contains the result of `compute_track_sizes` on the columns when sizing the rows. Used as an additional constraint for rows that size to content.
    fn compute_track_sizes(
        &self,
        layout_ctx: &mut LayoutCtx,
        parent_layout_constraints: &LayoutParams,
        env: &Environment,
        axis: Axis,
        tracks: &[TrackSize],
        track_count: usize,
        implicit_track_size: TrackSize,
        available_space: f64,
        row_gap: f64,
        column_gap: f64,
        column_layout: Option<&[GridTrackLayout]>,
    ) -> ComputeTrackSizeResult {
        let _span = trace_span!("grid track sizing", ?axis).entered();

        /*let tracks = match axis {
            TrackAxis::Row => &self.row_definitions[..],
            TrackAxis::Column => &self.column_definitions[..],
        };*/

        let gap = match axis {
            Axis::Row => row_gap,
            Axis::Column => column_gap,
        };

        let num_gutters = if track_count > 1 { track_count - 1 } else { 0 };

        let get_track_size = |i| {
            if i < tracks.len() {
                tracks[i]
            } else {
                implicit_track_size
            }
        };

        trace!("=== [{:?}] laying out: {:?} ===", self.widget_id(), axis);

        // base sizes (cross-axis) of the tracks (column widths, or row heights)
        let mut base_size = vec![0.0; track_count];
        let mut growth_limit = vec![0.0; track_count];

        // for each track, update base_size and growth limit
        for i in 0..track_count {
            trace!("--- laying out track {} ---", i);

            // If automatic sizing is requested (for min or max), compute the items natural sizes (result of layout with unbounded boxconstraints)
            // Also, for rows (axis == TrackAxis::Row) with AlignItems::Baseline, compute the max baseline offset of all items in the track
            let track_size = get_track_size(i);
            let auto_sized = track_size.min_size == TrackBreadth::Auto || track_size.max_size == TrackBreadth::Auto;
            let mut max_natural_size = 0.0f64;

            if auto_sized {
                let mut natural_layouts = Vec::new();
                for item in self.items_in_track(axis, i) {
                    // setup "unbounded" constraints, so that the child widget returns its "natural" size ...
                    let mut constraints = *parent_layout_constraints;
                    constraints.min.width = 0.0;
                    constraints.max.width = f64::INFINITY;
                    constraints.min.height = 0.0;
                    constraints.max.height = f64::INFINITY;

                    if let Some(column_layout) = column_layout {
                        // ... however, if we already determined the size of the columns,
                        // constrain the width by the size of the column range
                        let w = track_span_width(column_layout, item.column_range(), column_gap);
                        trace!("using column width constraint: max_width = {}", w);
                        constraints.max.width = w;
                    }

                    // get the "natural size" of the item under unbounded (or semi-bounded) constraints.
                    let natural_layout = item.widget.speculative_layout(layout_ctx, &constraints, env);
                    trace!("natural layout={:?}", natural_layout);
                    natural_layouts.push(natural_layout);
                }

                // calculate max baseline for items with baseline alignment
                let mut max_baseline = 0.0f64;
                for layout in natural_layouts.iter() {
                    if layout.y_align == Alignment::FirstBaseline {
                        max_baseline = max_baseline.max(layout.padding_box_baseline().unwrap_or(0.0));
                    }
                }

                // compute max element size (if necessary)
                for layout in natural_layouts.iter() {
                    let mut size = size_across(axis, layout.padding_box_size());
                    if axis == Axis::Row
                        && (layout.y_align == Alignment::FirstBaseline || layout.y_align == Alignment::LastBaseline)
                    {
                        // adjust the returned size with additional padding to account for baseline alignment
                        size += max_baseline - layout.padding_box_baseline().unwrap_or(0.0);
                    }
                    max_natural_size = max_natural_size.max(size);
                }

                trace!("max_natural_size={:?}", max_natural_size);
                trace!("max_baseline={:?}", max_baseline);

                trace!("track #{} max_natural_size={:?}", i, max_natural_size);
            }

            // apply min size constraint
            match track_size.min_size {
                TrackBreadth::Fixed(min) => {
                    // TODO width or height
                    base_size[i] = match axis {
                        Axis::Row => min.compute(parent_layout_constraints, env),
                        Axis::Column => min.compute(parent_layout_constraints, env),
                    };
                }
                TrackBreadth::Auto => {
                    base_size[i] = max_natural_size;
                }
                TrackBreadth::Flex(_) => {}
            };

            // apply max size constraint
            match track_size.max_size {
                TrackBreadth::Fixed(max) => {
                    growth_limit[i] = match axis {
                        Axis::Row => max.compute(parent_layout_constraints, env),
                        Axis::Column => max.compute(parent_layout_constraints, env),
                    };
                }
                TrackBreadth::Auto => {
                    // same as min size constraint
                    growth_limit[i] = max_natural_size;
                }
                TrackBreadth::Flex(_) => growth_limit[i] = f64::INFINITY,
            };

            if growth_limit[i] < base_size[i] {
                growth_limit[i] = base_size[i];
            }
        }

        // Maximize non-flex tracks, on the "free space", which is the available space minus
        // the space already taken by the fixed- and auto-sized element, and the gutter gaps.
        let mut free_space = available_space - base_size.iter().sum::<f64>() - (num_gutters as f64) * gap;
        for i in 0..track_count {
            // only maximize tracks with finite growth limits (otherwise flex tracks would take up all the space)
            if growth_limit[i].is_finite() {
                let delta = growth_limit[i] - base_size[i];
                if delta > 0.0 {
                    if free_space > delta {
                        base_size[i] = growth_limit[i];
                        free_space -= delta;
                    } else {
                        base_size[i] += free_space;
                        free_space = 0.0;
                        break;
                    }
                }
            }
        }

        // Distribute remaining spaces to flex tracks if the remaining free space is finite.
        // Otherwise they keep their assigned base sizes.
        if free_space.is_finite() {
            let mut flex_total = 0.0;
            for i in 0..track_count {
                let track_size = get_track_size(i);
                if let TrackBreadth::Flex(x) = track_size.max_size {
                    flex_total += x
                }
            }
            for i in 0..track_count {
                let track_size = get_track_size(i);
                if let TrackBreadth::Flex(x) = track_size.max_size {
                    let fr = x / flex_total;
                    base_size[i] = base_size[i].max(fr * free_space);
                }
            }
        }

        //tracing::trace!("{:?} base_size={:?}, growth_limit={:?}", axis, base_size, growth_limit);

        // grid line positions
        let mut layout = Vec::with_capacity(track_count);
        let mut pos = 0.0;
        for i in 0..base_size.len() {
            let size = base_size[i];
            //let baseline = baselines[i];
            layout.push(GridTrackLayout { pos, size });
            pos += size;
            if i != base_size.len() - 1 {
                pos += gap;
            }
        }

        ComputeTrackSizeResult { layout, size: pos }
    }

    /// Invalidates the cached child widget filter.
    fn invalidate_child_filter(&self) {
        self.cached_child_filter.set(None);
    }

    fn items_in_track(&self, axis: Axis, index: usize) -> impl Iterator<Item = &GridItem> {
        self.items.iter().filter(move |item| item.is_in_track(axis, index))
    }
}

impl Default for Grid {
    fn default() -> Self {
        Self::new(DEFAULT_GRID_TEMPLATE.clone())
    }
}

impl Widget for Grid {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutParams, env: &Environment) -> Geometry {
        let _span = trace_span!("grid layout", widget_id = ?WidgetId::dbg_option(self.widget_id())).entered();

        // TODO the actual direction of rows and columns depends on the writing mode
        // When (or if) we support other writing modes, rewrite this. Layout is complicated!

        // first, place items in the grid (i.e. resolve their grid areas into "definite areas")
        let (row_count, column_count) = self.position_items();

        // resolve styles
        let column_gap = self.style.column_gap.compute(constraints, env);
        let row_gap = self.style.row_gap.compute(constraints, env);
        let row_background = self.style.row_background.compute_paint(env);
        let alternate_row_background = self.style.alternate_row_background.compute_paint(env);
        let row_gap_background = self.style.row_gap_background.compute_paint(env);
        let column_gap_background = self.style.column_gap_background.compute_paint(env);

        // first measure the width of the columns
        let ComputeTrackSizeResult {
            layout: column_layout,
            size: width,
        } = self.compute_track_sizes(
            ctx,
            constraints,
            env,
            Axis::Column,
            &self.template.columns.sizes[..],
            column_count,
            TrackSize::new(self.implicit_column_size),
            constraints.max.width,
            row_gap,
            column_gap,
            None,
        );
        // then measure the height of the rows, which may depend on the width of the columns
        // Note: it may go the other way around (width of columns that depend on the height of the rows)
        // but we choose to do it like this
        let ComputeTrackSizeResult {
            layout: row_layout,
            size: height,
        } = self.compute_track_sizes(
            ctx,
            constraints,
            env,
            Axis::Row,
            &self.template.rows.sizes[..],
            row_count,
            TrackSize::new(self.implicit_row_size),
            constraints.max.height,
            row_gap,
            column_gap,
            Some(&column_layout[..]),
        );

        trace!("final row layout {:?}", row_layout);
        trace!("final column layout {:?}", column_layout);

        // --- measure the child items ---

        // (containing box size, child box layout)
        let mut child_layouts: Vec<(Size, Geometry)> = Vec::with_capacity(self.items.len());

        // Maximum horizontal baselines for each row of the grid (y-offset to the row's starting y-coordinate)
        let mut horizontal_baselines: Vec<f64> = vec![0.0; row_layout.len()];

        // maximum vertical baselines for each column of the grid (x-offset to the row's starting x-coordinate)
        // TODO implement vertical baselines & vertical baseline alignment
        //let mut vertical_baselines: Vec<f64> = vec![0.0; column_layout.len()];

        //trace!("=== START LAYOUT ===");

        {
            let _span = trace_span!("grid item measure").entered();
            for item in self.items.iter() {
                let (column_start, column_end) = item.column_range.get();
                let (row_start, row_end) = item.row_range.get();
                let w: f64 = track_span_width(&column_layout, column_start..column_end, column_gap);
                let h: f64 = track_span_width(&row_layout, row_start..row_end, row_gap);

                debug_assert!(
                    column_start < column_layout.len()
                        && column_end <= column_layout.len()
                        && row_start < row_layout.len()
                        && row_end <= row_layout.len()
                );

                let mut subconstraints = *constraints;
                subconstraints.max.width = w;
                subconstraints.max.height = h;
                subconstraints.min.width = 0.0;
                subconstraints.min.height = 0.0;

                let child_layout = item.widget.layout(ctx, &subconstraints, env);
                trace!(
                    "[{:?}] constraints: {:?}",
                    WidgetId::dbg_option(item.widget.widget_id()),
                    subconstraints
                );
                trace!(
                    "[{:?}] layout: {:?}",
                    WidgetId::dbg_option(item.widget.widget_id()),
                    child_layout
                );

                child_layouts.push((Size::new(w, h), child_layout));

                if child_layout.y_align == Alignment::FirstBaseline || child_layout.y_align == Alignment::LastBaseline {
                    // TODO last baseline
                    horizontal_baselines[row_start] =
                        horizontal_baselines[row_start].max(child_layout.measurements.baseline.unwrap_or(0.0));
                }
                // TODO vertical baselines
            }
        }

        {
            let _span = trace_span!("grid item placement").entered();
            // --- place items within their grid cells ---
            for (item, (containing_box_size, layout)) in self.items.iter().zip(child_layouts.iter()) {
                let (column_start, _column_end) = item.column_range.get();
                let (row_start, _row_end) = item.row_range.get();

                let cell_pos = Offset::new(column_layout[column_start].pos, row_layout[row_start].pos);
                let content_pos = layout.place_into(&Measurements {
                    size: *containing_box_size,
                    clip_bounds: None,
                    baseline: Some(horizontal_baselines[row_start]),
                });
                let offset = (cell_pos + content_pos).round_to_pixel(ctx.scale_factor);

                // TODO baselines...
                trace!(
                    "[{:?}] offset: {:?}",
                    WidgetId::dbg_option(item.widget.widget_id()),
                    offset
                );
                item.widget.set_offset(offset);
            }
        }
        // trace!("=== END LAYOUT ===");

        // ------ update cache ------
        self.computed.set(Arc::new(Computed {
            row_layout,
            column_layout,
            style: ComputedStyle {
                row_gap,
                column_gap,
                row_background,
                alternate_row_background,
                row_gap_background,
                column_gap_background,
            },
            width,
            height,
            show_grid_lines: env.get(&SHOW_GRID_LAYOUT_LINES).unwrap_or_default(),
        }));

        // TODO baseline
        Geometry::new(Size::new(width, height))
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        //if let Event::MoveFocus()

        // run the events through the items in reverse order
        // in order to give priority to topmost items
        for item in self.items.iter().rev() {
            item.widget.route_event(ctx, event, env);
        }

        // if ctx.move_focus_requested() {
        //
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        use skia_safe as sk;
        let height = ctx.bounds.size.height;
        let width = ctx.bounds.size.width;

        let computed = self.computed.get();
        let row_layout = &computed.row_layout;
        let column_layout = &computed.column_layout;

        // draw row backgrounds
        if !computed.style.row_background.is_transparent() && !computed.style.alternate_row_background.is_transparent()
        {
            for (i, row) in row_layout.iter().enumerate() {
                // TODO start index
                let bg = if i % 2 == 0 {
                    computed.style.row_background.clone()
                } else {
                    computed.style.alternate_row_background.clone()
                };
                ctx.fill_shape(
                    &Shape::from(Rect::new(Point::new(0.0, row.pos), Size::new(width, row.size))),
                    &bg,
                );
            }
        }

        // draw gap backgrounds
        if !computed.style.row_gap_background.is_transparent() {
            // draw only inner gaps
            for row in row_layout.iter().skip(1) {
                ctx.fill_shape(
                    &Shape::from(Rect::new(
                        Point::new(0.0, row.pos - computed.style.row_gap),
                        Size::new(width, computed.style.row_gap),
                    )),
                    &computed.style.row_gap_background,
                );
            }
        }
        if !computed.style.column_gap_background.is_transparent() {
            for column in column_layout.iter().skip(1) {
                ctx.fill_shape(
                    &Shape::from(Rect::new(
                        Point::new(column.pos - computed.style.column_gap, 0.0),
                        Size::new(computed.style.column_gap, height),
                    )),
                    &computed.style.column_gap_background,
                );
            }
        }

        // draw elements
        for item in self.items.iter() {
            item.widget.paint(ctx);
        }

        /*// draw debug grid lines
        if computed.show_grid_lines {
            let paint = sk::Paint::new(Color::new(1.0, 0.5, 0.2, 1.0).to_skia(), None);
            for x in column_layout.iter().map(|x| x.pos).chain(std::iter::once(width - 1.0)) {
                ctx.surface.canvas().draw_line(
                    Point::new(x + 0.5, 0.5).to_skia(),
                    Point::new(x + 0.5, height + 0.5).to_skia(),
                    &paint,
                );
            }
            for y in row_layout.iter().map(|x| x.pos).chain(std::iter::once(height - 1.0)) {
                ctx.surface.canvas().draw_line(
                    Point::new(0.5, y + 0.5).to_skia(),
                    Point::new(width + 0.5, y + 0.5).to_skia(),
                    &paint,
                );
            }
        }*/
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!(
            "{} by {} grid",
            self.template_row_count(),
            self.template_column_count()
        ))
    }
}
