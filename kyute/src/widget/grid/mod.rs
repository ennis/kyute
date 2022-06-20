use crate::{
    bloom::Bloom,
    cache,
    core::DebugNode,
    drawing::ToSkia,
    style::{Paint, PaintCtxExt, Style},
    widget::prelude::*,
    Color, Data, EnvKey, EnvRef, GpuFrameCtx, InternalEvent, Length, PointerEventKind, RoundToPixel, State,
    WidgetFilter, WidgetId,
};
use cssparser::{ParseError, Parser, Token};
use lazy_static::lazy_static;
use std::{
    cell::{Cell, RefCell},
    cmp::{max, min},
    collections::HashMap,
    convert::{TryFrom, TryInto},
    mem,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    sync::Arc,
};
use svgtypes::Align;

pub const SHOW_GRID_LAYOUT_LINES: EnvKey<bool> = EnvKey::new("kyute.show_grid_layout_lines");

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

/// Orientation of a grid track.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Axis {
    /// Row (or _inline_) axis (follows the text).
    Row,
    /// Column (or _block_) axis, perpendicular to the flow of text.
    Column,
}

/// Returns the size of a box along the specified axis.
fn size_along(axis: Axis, size: Size) -> f64 {
    // TODO depends on the writing mode
    match axis {
        Axis::Row => size.width,
        Axis::Column => size.height,
    }
}

/// Returns the size of a box along the specified axis.
fn size_across(axis: Axis, size: Size) -> f64 {
    // TODO depends on the writing mode
    match axis {
        Axis::Row => size.height,
        Axis::Column => size.width,
    }
}

/// List of tracks.
#[derive(Clone, Debug)]
pub struct TrackList {
    sizes: Vec<TrackSize>,
    line_names: Vec<(usize, String)>,
}

fn grid_line_names<'i>(input: &mut Parser<'i, '_>) -> Result<Vec<String>, ParseError<'i, ()>> {
    input.expect_square_bracket_block()?;
    input.parse_nested_block(|input| {
        let idents = input.parse_comma_separated(Parser::expect_ident)?;
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
        let rows = TrackList::parse_impl(input)?;
        let columns = TrackList::parse_impl(input)?;
        Ok(GridTemplate { rows, columns })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Line / LineRange
////////////////////////////////////////////////////////////////////////////////////////////////////

/// Identifies a particular grid line or a line span.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Line<'a> {
    Auto,
    /// Identifies a line by its name, as defined in the grid template.
    Named(&'a str),
    /// Identifies a line by its index.
    Index(i32),
    Span(usize),
}

impl<'a> Default for Line<'a> {
    fn default() -> Self {
        Line::Auto
    }
}

impl<'a> From<i32> for Line<'a> {
    fn from(p: i32) -> Self {
        Line::Index(p)
    }
}

impl<'a> From<&'a str> for Line<'a> {
    fn from(s: &'a str) -> Self {
        Line::Named(s)
    }
}

impl<'a> Line<'a> {
    /// Parses a <grid-line> CSS value.
    pub(crate) fn parse_css(input: &mut Parser<'a, '_>) -> Result<Line<'a>, ParseError<'a, ()>> {
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
            (Ok(Token::Ident(id)), Err(_)) => Ok(Line::Named(&**id)),
            _ => Err(input.new_custom_error(())),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//  LineRange
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct LineRange<'a> {
    pub start: Line<'a>,
    pub end: Line<'a>,
}

impl<'a> From<Line<'a>> for LineRange<'a> {
    fn from(start: Line<'a>) -> Self {
        LineRange {
            start,
            end: Line::Span(1),
        }
    }
}

impl<'a> From<i32> for LineRange<'a> {
    fn from(p: i32) -> Self {
        LineRange {
            start: Line::Index(p),
            end: Line::Span(1),
        }
    }
}

impl<'a> From<usize> for LineRange<'a> {
    fn from(p: usize) -> Self {
        LineRange {
            start: Line::Index(p as i32),
            end: Line::Span(1),
        }
    }
}

impl<'a> From<Range<i32>> for LineRange<'a> {
    fn from(v: Range<i32>) -> Self {
        LineRange {
            start: Line::Index(v.start),
            end: Line::Index(v.end),
        }
    }
}

impl<'a> From<Range<usize>> for LineRange<'a> {
    fn from(v: Range<usize>) -> Self {
        LineRange {
            start: Line::Index(v.start as i32),
            end: Line::Index(v.end as i32),
        }
    }
}

impl<'a> From<RangeTo<i32>> for LineRange<'a> {
    fn from(v: RangeTo<i32>) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index(v.end),
        }
    }
}

impl<'a> From<RangeFrom<i32>> for LineRange<'a> {
    fn from(v: RangeFrom<i32>) -> Self {
        LineRange {
            start: Line::Index(v.start),
            end: Line::Index(-1),
        }
    }
}

impl<'a> From<RangeInclusive<i32>> for LineRange<'a> {
    fn from(v: RangeInclusive<i32>) -> Self {
        LineRange {
            start: Line::Index(*v.start()),
            end: Line::Index((*v.end() + 1) as i32),
        }
    }
}

impl<'a> From<RangeInclusive<usize>> for LineRange<'a> {
    fn from(v: RangeInclusive<usize>) -> Self {
        LineRange {
            start: Line::Index(*v.start() as i32),
            end: Line::Index((*v.end() + 1) as i32),
        }
    }
}

impl<'a> From<RangeToInclusive<i32>> for LineRange<'a> {
    fn from(v: RangeToInclusive<i32>) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index(v.end + 1),
        }
    }
}

impl<'a> From<RangeToInclusive<usize>> for LineRange<'a> {
    fn from(v: RangeToInclusive<usize>) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index((v.end + 1) as i32),
        }
    }
}

impl<'a> From<RangeFull> for LineRange<'a> {
    fn from(_: RangeFull) -> Self {
        LineRange {
            start: Line::Index(0),
            end: Line::Index(-1),
        }
    }
}

impl<'a> TryFrom<&'a str> for LineRange<'a> {
    type Error = nom::error::Error<String>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        LineRange::parse(input)
    }
}

impl<'a> LineRange<'a> {
    /// Parses the value of a `grid-row` or `grid-column` property declaration.
    pub(crate) fn parse_impl(input: &mut Parser<'a, '_>) -> Result<LineRange<'a>, ParseError<'a, ()>> {
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

impl<'a> LineRange<'a> {
    fn resolve(&self, named_lines: &HashMap<String, usize>, line_count: usize) -> (Option<usize>, usize) {
        if let (Line::Span(_), Line::Span(_)) = (self.start, self.end) {
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
            Line::Named(ident) => {
                start = named_lines.get(ident).cloned();
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
            Line::Named(ident) => {
                end = named_lines.get(ident).cloned();
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
#[derive(Copy, Default, Clone, Debug, PartialEq, Eq)]
pub struct Area<'a> {
    row: LineRange<'a>,
    column: LineRange<'a>,
}

/*impl<'a> TryFrom<&'a str> for Area<'a> {
    type Error = nom::error::Error<String>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        Area::parse(input)
    }
}*/

impl<'a, Rows, Columns> From<(Rows, Columns)> for Area<'a>
where
    Rows: Into<LineRange<'a>>,
    Columns: Into<LineRange<'a>>,
{
    fn from((rows, columns): (Rows, Columns)) -> Self {
        Area {
            row: rows.into(),
            column: columns.into(),
        }
    }
}

impl<'a> Area<'a> {
    /// Parses the value of a `grid-area` CSS property.
    pub(crate) fn parse_impl(input: &mut Parser<'a, '_>) -> Result<Area<'a>, ParseError<'a, ()>> {
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

#[derive(Copy, Clone, Debug)]
struct DefiniteArea {
    /// If None, use flow
    row: Option<usize>,
    column: Option<usize>,
    row_span: usize,
    column_span: usize,
}

impl<'a> Area<'a> {
    fn resolve(&self, grid: &Grid) -> DefiniteArea {
        let (row, row_span) = self.row.resolve(&grid.named_row_lines, grid.row_count() + 1);
        let (column, column_span) = self.column.resolve(&grid.named_column_lines, grid.column_count() + 1);

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

pub trait Insertable {
    fn insert(self, cursor: &mut FlowCursor);
}

impl<W> Insertable for W
where
    W: Widget + Sized + 'static,
{
    fn insert(self, cursor: &mut FlowCursor) {
        cursor.place(Area::default(), 0, Alignment::TOP_LEFT, Arc::new(WidgetPod::new(self)))
    }
}

impl Insertable for () {
    fn insert(self, cursor: &mut FlowCursor) {
        cursor.next(1, 1);
    }
}

macro_rules! tuple_insertable {
    () => {};
    ( $w:ident : $t:ident, $($ws:ident : $ts:ident, )* ) => {
        impl<$t, $($ts,)*> Insertable for ($t, $($ts,)* ) where
            $t: Insertable + 'static,
            $( $ts: Insertable + 'static ),*
        {
            fn insert(self, cursor: &mut FlowCursor)
            {
                let ($w, $($ws,)*) = self;
                $w.insert(cursor);
                $($ws.insert(cursor);)*
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

pub struct GridPlacer<'a, W> {
    area: Area<'a>,
    alignment: Alignment,
    widget: W,
}

impl<'a, W> GridPlacer<'a, W> {
    pub fn new(widget: W) -> GridPlacer<'a, W> {
        GridPlacer {
            area: Default::default(),
            alignment: Alignment::TOP_LEFT,
            widget,
        }
    }

    pub fn grid_row_start(mut self, line: impl TryInto<Line<'a>>) -> Self {
        self.area.row.start = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_row_end(mut self, line: impl TryInto<Line<'a>>) -> Self {
        self.area.row.end = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_column_start(mut self, line: impl TryInto<Line<'a>>) -> Self {
        self.area.column.start = line.try_into().unwrap_or_default();
        self
    }

    pub fn grid_column_end(mut self, line: impl TryInto<Line<'a>>) -> Self {
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

    pub fn grid_row(mut self, range: impl TryInto<LineRange<'a>>) -> Self {
        self.area.row = range.try_into().unwrap_or_default();
        self
    }

    pub fn grid_column(mut self, range: impl TryInto<LineRange<'a>>) -> Self {
        self.area.column = range.try_into().unwrap_or_default();
        self
    }

    pub fn grid_area(mut self, area: impl TryInto<Area<'a>>) -> Self {
        self.area = area.try_into().unwrap_or_default();
        self
    }
}

impl<'a, W> Insertable for GridPlacer<'a, W>
where
    W: Widget + 'static,
{
    fn insert(self, cursor: &mut FlowCursor) {
        cursor.place(self.area, 1, self.alignment, Arc::new(WidgetPod::new(self.widget)));
    }
}

pub trait GridLayoutExt: Widget + Sized {
    fn grid_row_start<'a>(self, line: impl TryInto<Line<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_row_start(line)
    }

    fn grid_row_end<'a>(self, line: impl TryInto<Line<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_row_start(line)
    }

    fn grid_column_start<'a>(self, line: impl TryInto<Line<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_column_start(line)
    }

    fn grid_column_end<'a>(self, line: impl TryInto<Line<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_column_end(line)
    }

    fn grid_row_span<'a>(self, len: usize) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_row_span(len)
    }

    fn grid_column_span<'a>(self, len: usize) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_column_span(len)
    }

    fn grid_row<'a>(self, range: impl TryInto<LineRange<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_row(range)
    }

    fn grid_column<'a>(self, range: impl TryInto<LineRange<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_column(range)
    }

    fn grid_area<'a>(self, area: impl TryInto<Area<'a>>) -> GridPlacer<'a, Self> {
        GridPlacer::new(self).grid_area(area)
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
    row_range: Cell<Range<usize>>,
    column_range: Cell<Range<usize>>,
    z_order: i32,
    widget: Arc<WidgetPod>,
    // only used for "degenerate" row/col spans
    line_alignment: Alignment,
}

impl GridItem {
    fn is_in_track(&self, axis: Axis, index: usize) -> bool {
        // "grid line" items (those with row_range.len() == 0 or column_range.len() == 0)
        // are not considered to belong to any track, and don't intervene during track sizing
        if self.row_range.is_empty() || self.column_range.is_empty() {
            return false;
        }
        match axis {
            Axis::Row => self.row_range.start == index,
            Axis::Column => self.column_range.start == index,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridTrackLayout {
    pub pos: f64,
    pub size: f64,
    //pub baseline: Option<f64>,
}

struct ComputeTrackSizeResult {
    layout: Vec<GridTrackLayout>,
    size: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct GridLayout {
    row_layout: Vec<GridTrackLayout>,
    column_layout: Vec<GridTrackLayout>,
    row_gap: f64,
    column_gap: f64,
    width: f64,
    height: f64,
    show_grid_lines: bool,
}

impl Data for GridLayout {
    fn same(&self, other: &Self) -> bool {
        self == other
    }
}

/// Visual style of a grid layout container.
#[derive(Clone, Debug, Default)]
pub struct GridStyle {
    pub row_gap: Length,
    pub column_gap: Length,
    //pub align_items: AlignItems,
    //pub justify_items: JustifyItems,
    /// Row background.
    pub row_background: Paint,
    /// Alternate row background.
    pub alternate_row_background: Paint,
    /// Row gap background.
    pub row_gap_background: Paint,
    /// Column gap background.
    pub column_gap_background: Paint,
}

/// Grid layout container.
///
/// TODO it's a bit heavyweight for just layouting two buttons in a column...
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
    calculated_layout: State<Arc<GridLayout>>,
    cached_child_filter: Cell<Option<Bloom<WidgetId>>>,
}

/// Returns the size of a column span
fn track_span_width(layout: &[GridTrackLayout], span: Range<usize>, gap: f64) -> f64 {
    layout[span.clone()].iter().map(|x| x.size).sum::<f64>() + gap * (span.len() as isize - 1).max(0) as f64
}

lazy_static! {
    static ref DEFAULT_GRID_STYLE: Arc<GridStyle> = Arc::new(GridStyle::default());
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
            style: DEFAULT_GRID_STYLE.clone(),
            calculated_layout: cache::state(|| Default::default()),
            cached_child_filter: Cell::new(None),
        }
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

    pub fn set_row_background(&mut self, row_background: impl Into<Paint>) {
        Arc::make_mut(&mut self.style).row_background = row_background.into();
    }

    pub fn set_alternate_row_background(&mut self, alternate_row_background: impl Into<Paint>) {
        Arc::make_mut(&mut self.style).alternate_row_background = alternate_row_background.into();
    }

    pub fn set_row_gap_background(&mut self, bg: impl Into<Paint>) {
        Arc::make_mut(&mut self.style).row_gap_background = bg.into();
    }

    pub fn set_column_gap_background(&mut self, bg: impl Into<Paint>) {
        Arc::make_mut(&mut self.style).column_gap_background = bg.into();
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
enum FlowDirection {
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
            let (row_range, column_range) = flow_cursor.place(item.area);
            final_row_count = final_row_count.max(row_range.end);
            final_column_count = final_column_count.max(column_range.end);
            item.row_range.set(row_range);
            item.column_range.set(column_range);
        }

        (final_row_count, final_column_count)
    }

    /// Computes the sizes of rows or columns.
    ///
    /// * `available_space`: max size across track direction (columns => max width, rows => max height).
    /// * `column_sizes`: contains the result of `compute_track_sizes` on the columns when sizing the rows. Used as an additional constraint for rows that size to content.
    fn compute_track_sizes(
        &self,
        layout_ctx: &mut LayoutCtx,
        parent_layout_constraints: &LayoutConstraints,
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
            if i < track_count {
                tracks[i]
            } else {
                implicit_track_size
            }
        };

        // base sizes (cross-axis) of the tracks (column widths, or row heights)
        let mut base_size = vec![0.0; track_count];
        let mut growth_limit = vec![0.0; track_count];

        // for each track, update base_size and growth limit
        for i in 0..track_count {
            // If automatic sizing is requested (for min or max), compute the items natural sizes (result of layout with unbounded boxconstraints)
            // Also, for rows (axis == TrackAxis::Row) with AlignItems::Baseline, compute the max baseline offset of all items in the track
            let track_size = get_track_size(i);
            let auto_sized = track_size.min_size == TrackBreadth::Auto || track_size.max_size == TrackBreadth::Auto;
            let mut max_natural_size = 0.0;

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
                        let w = track_span_width(column_layout, item.column_range.get().clone(), column_gap);
                        constraints.max.width = w;
                    }

                    // get the "natural size" of the item under unbounded (or semi-bounded) constraints.
                    let natural_layout = item.widget.speculative_layout(layout_ctx, &constraints, env);
                    natural_layouts.push(natural_layout);
                }

                // calculate max baseline for items with baseline alignment
                let mut max_baseline = 0.0;
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
            }

            // apply min size constraint
            match track_size.min_size {
                TrackBreadth::Fixed(min) => {
                    // TODO width or height
                    base_size[i] = match axis {
                        Axis::Row => parent_layout_constraints.resolve_height(min),
                        Axis::Column => parent_layout_constraints.resolve_width(min),
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
                        Axis::Row => parent_layout_constraints.resolve_height(max),
                        Axis::Column => parent_layout_constraints.resolve_width(max),
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

    fn layout(&self, ctx: &mut LayoutCtx, constraints: &LayoutConstraints, env: &Environment) -> Measurements {
        // TODO the actual direction of rows and columns depends on the writing mode
        // When (or if) we support other writing modes, rewrite this. Layout is complicated!

        // place items
        let (row_count, column_count) = self.position_items();

        // compute gap sizes
        let column_gap = constraints.resolve_width(self.style.column_gap);
        let row_gap = constraints.resolve_height(self.style.row_gap);

        trace!("grid: recomputing track sizes");
        // no match, recalculate
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

        // layout items
        for item in self.items.iter() {
            let w: f64 = track_span_width(&column_layout, item.column_range.get().clone(), column_gap);
            let h: f64 = track_span_width(&row_layout, item.row_range.get().clone(), row_gap);

            let mut subconstraints = *constraints;
            subconstraints.max.width = w;
            subconstraints.max.height = h;
            subconstraints.min.width = 0.0;
            subconstraints.min.height = 0.0;
            let sublayout = item.widget.layout(ctx, constraints, env);

            let offset = sublayout.content_box_offset(Size::new(w, h));
            if sublayout.y_align == Alignment::FirstBaseline || sublayout.y_align == Alignment::LastBaseline {
                // TODO
            }

            let x = column_layout[item.column_range.start].pos;
            let y = row_layout[item.row_range.start].pos;
            let cell_offset = Offset::new(x, y);

            // TODO baselines...
            item.widget
                .set_offset((cell_offset + offset).round_to_pixel(ctx.scale_factor));
        }

        // ------ update cache ------
        self.calculated_layout.set(Arc::new(GridLayout {
            row_layout,
            column_layout,
            row_gap,
            column_gap,
            width,
            height,
            show_grid_lines: env.get(SHOW_GRID_LAYOUT_LINES).unwrap_or_default(),
        }));

        // TODO baseline
        Measurements::new(Size::new(width, height))
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // run the events through the items in reverse order
        // in order to give priority to topmost items
        for item in self.items.iter().rev() {
            item.widget.route_event(ctx, event, env);
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        use skia_safe as sk;
        let height = ctx.bounds.size.height;
        let width = ctx.bounds.size.width;

        let layout = self.calculated_layout.get();
        let row_layout = &layout.row_layout;
        let column_layout = &layout.column_layout;

        // draw row backgrounds
        if !self.row_background.is_transparent() && !self.alternate_row_background.is_transparent() {
            for (i, row) in row_layout.iter().enumerate() {
                // TODO start index
                let bg = if i % 2 == 0 {
                    self.row_background.clone()
                } else {
                    self.alternate_row_background.clone()
                };
                ctx.draw_styled_box(
                    Rect::new(Point::new(0.0, row.pos), Size::new(width, row.size)),
                    &Style::new().background(bg),
                );
            }
        }

        // draw gap backgrounds
        if !self.row_gap_background.is_transparent() {
            // draw only inner gaps
            for row in row_layout.iter().skip(1) {
                ctx.draw_styled_box(
                    Rect::new(
                        Point::new(0.0, row.pos - layout.row_gap),
                        Size::new(width, layout.row_gap),
                    ),
                    &Style::new().background(self.row_gap_background.clone()),
                );
            }
        }
        if !self.column_gap_background.is_transparent() {
            for column in column_layout.iter().skip(1) {
                ctx.draw_styled_box(
                    Rect::new(
                        Point::new(column.pos - layout.column_gap, 0.0),
                        Size::new(layout.column_gap, height),
                    ),
                    &Style::new().background(self.column_gap_background.clone()),
                );
            }
        }

        // draw elements
        for item in self.items.iter() {
            item.widget.paint(ctx);
        }

        // draw debug grid lines
        if layout.show_grid_lines {
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
        }
    }

    fn debug_node(&self) -> DebugNode {
        DebugNode::new(format!("{} by {} grid", self.row_count(), self.column_count()))
    }
}
