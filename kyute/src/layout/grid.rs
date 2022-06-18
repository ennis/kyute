//! Grid layout implementation.
use crate::{
    bloom::Bloom,
    cache,
    core::DebugNode,
    drawing::ToSkia,
    style,
    style::{Paint, PaintCtxExt, Style},
    widget::prelude::*,
    Color, Data, EnvKey, EnvRef, GpuFrameCtx, InternalEvent, Length, PointerEventKind, RoundToPixel, State,
    WidgetFilter, WidgetId,
};

use crate::style::StyleCtx;
use kyute::style::ToComputedValue;
use std::{
    cell::{Cell, RefCell},
    cmp::{max, min},
    collections::HashMap,
    convert::{TryFrom, TryInto},
    mem,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    sync::Arc,
};

/// Specifies alignment along the inline axis.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Data)]
pub enum JustifyItems {
    Start,
    End,
    Center,
    Stretch,
}

/// Specifies alignment along the block axis (perpendicular to the text).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Data)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

/// Orientation of a grid track.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum TrackAxis {
    Row,
    Column,
}

impl TrackAxis {
    /// Width for a column, height for a row
    fn width(&self, size: Size) -> f64 {
        match self {
            TrackAxis::Column => size.width,
            TrackAxis::Row => size.height,
        }
    }
}

/// An item inserted into a grid
#[derive(Clone, Debug)]
struct GridItem {
    row_range: Range<usize>,
    column_range: Range<usize>,
    z_order: i32,
    widget: Arc<WidgetPod>,
    // only used for "degenerate" row/col spans
    line_alignment: Alignment,
}

impl GridItem {
    /*fn track_span(&self, axis: TrackAxis) -> Range<usize> {
        match axis {
            TrackAxis::Row => self.row_range.clone(),
            TrackAxis::Column => self.column_range.clone(),
        }
    }*/

    fn is_in_track(&self, axis: TrackAxis, index: usize) -> bool {
        // "grid line" items (those with row_range.len() == 0 or column_range.len() == 0)
        // are not considered to belong to any track, and don't intervene during track sizing
        if self.row_range.is_empty() || self.column_range.is_empty() {
            return false;
        }
        match axis {
            TrackAxis::Row => self.row_range.start == index,
            TrackAxis::Column => self.column_range.start == index,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridTrackLayout {
    pub pos: f64,
    pub size: f64,
    pub baseline: Option<f64>,
}

struct ComputeTrackSizeResult {
    layout: Vec<GridTrackLayout>,
    size: f64,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// GridTemplate
////////////////////////////////////////////////////////////////////////////////////////////////////

/// A template for a grid's rows, columns, and gaps.
#[derive(Default, Debug)]
pub struct GridTemplate {
    pub rows: Vec<TrackSize>,
    pub columns: Vec<TrackSize>,
    pub row_tags: Vec<(usize, String)>,
    pub column_tags: Vec<(usize, String)>,
    pub implicit_row_size: TrackSize,
    pub implicit_column_size: TrackSize,
    pub row_gap: Option<Length>,
    pub column_gap: Option<Length>,
}

impl TryFrom<&str> for GridTemplate {
    type Error = nom::error::Error<String>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        GridTemplate::parse(value)
    }
}

impl GridTemplate {
    pub fn new() -> GridTemplate {
        GridTemplate::default()
    }

    pub fn push_row(&mut self, size: impl Into<TrackSize>) {
        self.rows.push(size.into());
    }

    pub fn push_row_tag(&mut self, tag: impl Into<String>) {
        self.row_tags.push((self.rows.len(), tag.into()));
    }

    pub fn push_column(&mut self, size: impl Into<TrackSize>) {
        self.columns.push(size.into());
    }

    pub fn push_column_tag(&mut self, tag: impl Into<String>) {
        self.column_tags.push((self.columns.len(), tag.into()));
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

impl<'a> TryFrom<&'a str> for LineRange {
    type Error = nom::error::Error<String>;

    fn try_from(input: &'a str) -> Result<Self, Self::Error> {
        LineRange::parse(input)
    }
}

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

impl LineRange {
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

impl ToComputedValue for LineRange {
    type ComputedValue = LineRange;

    fn to_computed_value(&self, context: &StyleCtx) -> LineRange {
        self.clone()
    }
}

///////////////////////////////////////////////

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

#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum FlowDirection {
    /// Fill rows first
    Row,
    /// Fill colums first
    Column,
}

#[derive(Debug)]
pub struct FlowCursor<'a> {
    grid: &'a mut Grid,
    row: usize,
    column: usize,
    row_len: usize,
    flow: FlowDirection,
}

impl<'a> FlowCursor<'a> {
    pub fn align(&mut self, column: usize) {
        if self.column < column {
            self.column = column;
        } else if self.column > column {
            self.row += 1;
            self.column = column;
        }
    }

    pub fn next(&mut self, row_span: usize, column_span: usize) -> (usize, usize) {
        let (row, column) = (self.row, self.column);
        self.column += column_span;
        if self.column >= self.row_len {
            self.row += row_span;
            self.column = 0;
        }
        (row, column)
    }

    fn place_helper(
        &mut self,
        row: usize,
        row_span: usize,
        column: usize,
        column_span: usize,
        z_order: i32,
        alignment: Alignment,
        widget: Arc<WidgetPod>,
    ) {
        match self.flow {
            FlowDirection::Row => self.grid.place_inner(
                row..(row + row_span),
                column..(column + column_span),
                z_order,
                alignment,
                widget,
            ),
            FlowDirection::Column => self.grid.place_inner(
                column..(column + column_span),
                row..(row + row_span),
                z_order,
                alignment,
                widget,
            ),
        }
    }

    pub fn place(&mut self, at: Area, z_order: i32, alignment: Alignment, widget: Arc<WidgetPod>) {
        let area = at.resolve(self.grid);

        let mut row = area.row;
        let mut column = area.column;
        let mut row_span = area.row_span;
        let mut column_span = area.column_span;

        if self.flow == FlowDirection::Column {
            mem::swap(&mut row, &mut column);
            mem::swap(&mut row_span, &mut column_span);
        }

        match (row, column) {
            (Some(row), Some(column)) => {
                self.place_helper(row, row_span, column, column_span, z_order, alignment, widget);
            }
            (Some(row), None) => {
                // TODO packing
                self.place_helper(row, row_span, 0, column_span, z_order, alignment, widget);
            }
            (None, col) => {
                if let Some(column) = col {
                    self.align(column);
                }
                let (row, column) = self.next(row_span, column_span);
                self.place_helper(row, row_span, column, column_span, z_order, alignment, widget);
            }
        }
    }
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

pub struct GridLayoutParams {}

/// Entry point of the grid layout algorithm.
///
///
pub fn grid_layout(layout_ctx: &mut LayoutCtx, style: &style::ComputedValues, items: &mut [Vec<Arc<WidgetPod>>]) {
    let template_rows = &style.grid.template_rows;
    let template_columns = &style.grid.template_columns;
    let row_gap = style.grid.row_gap;
    let column_gap = style.grid.column_gap;

    let child_layouts = items.iter_mut().map(|child| child.layout());
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

// For each inserted item:
// - explicit position OR flow options
// - row span / column span
// - cell or line positioning
// - flow options:
//      - align with column / row (line in flow direction)

/// Grid layout container.
///
/// TODO it's a bit heavyweight for just layouting two buttons in a column...
#[derive(Debug)]
pub struct Grid {
    id: WidgetId,
    /// Column sizes.
    column_definitions: Vec<TrackSize>,
    /// Row sizes.
    row_definitions: Vec<TrackSize>,

    /// List of grid items: widgets positioned inside the grid.
    items: Vec<GridItem>,

    named_row_lines: HashMap<String, usize>,
    named_column_lines: HashMap<String, usize>,

    /// Row template.
    implicit_row_size: GridLength,
    implicit_column_size: GridLength,
    row_gap: Length,
    column_gap: Length,

    align_items: AlignItems,
    justify_items: JustifyItems,

    auto_flow_dir: FlowDirection,
    auto_flow_row: usize,
    auto_flow_col: usize,

    // style
    /// Row background.
    row_background: Paint,
    /// Alternate row background.
    alternate_row_background: Paint,

    /// Row gap background.
    row_gap_background: Paint,
    /// Column gap background.
    column_gap_background: Paint,

    ///
    calculated_layout: State<Arc<GridLayout>>,
    cached_child_filter: Cell<Option<Bloom<WidgetId>>>,
}

/// Returns the size of a column span
fn track_span_width(layout: &[GridTrackLayout], span: Range<usize>, gap: f64) -> f64 {
    layout[span.clone()].iter().map(|x| x.size).sum::<f64>() + gap * (span.len() as isize - 1).max(0) as f64
}

impl Grid {
    /// Creates a new grid, initially without any row or column definitions.
    pub fn new() -> Grid {
        Grid {
            id: WidgetId::here(),
            column_definitions: vec![],
            row_definitions: vec![],
            items: vec![],
            named_row_lines: HashMap::default(),
            named_column_lines: HashMap::default(),
            implicit_row_size: GridLength::Auto,
            implicit_column_size: GridLength::Auto,
            row_gap: Length::Dip(0.0),
            column_gap: Length::Dip(0.0),
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            auto_flow_dir: FlowDirection::Row,
            auto_flow_row: 0,
            auto_flow_col: 0,
            row_background: Default::default(),
            alternate_row_background: Default::default(),
            row_gap_background: Default::default(),
            column_gap_background: Default::default(),
            calculated_layout: cache::state(|| Default::default()),
            cached_child_filter: Cell::new(None),
        }
    }

    /// Creates a new grid from a template.
    pub fn with_template(template: impl TryInto<GridTemplate>) -> Grid {
        let template = template.try_into().unwrap_or_else(|err| {
            warn!("invalid grid template");
            Default::default()
        });
        let mut grid = Self::new();
        grid.row_definitions = template.rows;
        grid.column_definitions = template.columns;

        // TODO
        grid.implicit_row_size = template.implicit_row_size.min_size;
        // TODO
        grid.implicit_column_size = template.implicit_column_size.min_size;

        for (row, tag) in template.row_tags {
            grid.named_row_lines.insert(tag, row);
        }
        for (column, tag) in template.column_tags {
            grid.named_column_lines.insert(tag, column);
        }

        grid.row_gap = template.row_gap.unwrap_or_default();
        grid.column_gap = template.column_gap.unwrap_or_default();
        grid
    }

    /// Creates a single-column grid.
    pub fn column(width: impl Into<GridLength>) -> Grid {
        let mut grid = Self::new();
        grid.column_definitions.push(TrackSize::new(width));
        grid
    }

    /// Creates a single-row grid.
    pub fn row(height: impl Into<GridLength>) -> Grid {
        let mut grid = Self::new();
        grid.row_definitions.push(TrackSize::new(height));
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

    /// Returns the current number of rows
    pub fn row_count(&self) -> usize {
        self.row_definitions.len()
    }

    /// Returns the current number of columns
    /// FIXME this should return the number of columns in the template
    pub fn column_count(&self) -> usize {
        self.column_definitions.len()
    }

    /// Sets the size of the gap between rows.
    pub fn set_row_gap(&mut self, gap: impl Into<Length>) {
        self.row_gap = gap.into();
    }

    /// Sets the size of the gap between columns.
    pub fn set_column_gap(&mut self, gap: impl Into<Length>) {
        self.column_gap = gap.into();
    }

    /*/// Sets the template for implicit column definitions.
    pub fn column_template(mut self, size: GridLength) -> Self {
        self.column_template = size;
        self
    }*/

    pub fn set_align_items(&mut self, align_items: AlignItems) {
        self.align_items = align_items;
    }

    pub fn set_justify_items(&mut self, justify_items: JustifyItems) {
        self.justify_items = justify_items;
    }

    pub fn set_row_background(&mut self, row_background: impl Into<Paint>) {
        self.row_background = row_background.into();
    }

    pub fn set_alternate_row_background(&mut self, alternate_row_background: impl Into<Paint>) {
        self.alternate_row_background = alternate_row_background.into();
    }

    pub fn set_row_gap_background(&mut self, bg: impl Into<Paint>) {
        self.row_gap_background = bg.into();
    }

    pub fn set_column_gap_background(&mut self, bg: impl Into<Paint>) {
        self.column_gap_background = bg.into();
    }

    #[composable]
    fn place_inner<'a>(
        &mut self,
        row_range: Range<usize>,
        column_range: Range<usize>,
        z_order: i32,
        line_alignment: Alignment,
        widget: Arc<WidgetPod>,
    ) {
        let is_grid_line = row_range.is_empty() || column_range.is_empty();

        // add rows/columns as required
        let num_rows;
        let num_columns;
        if is_grid_line {
            // N+1 grid lines
            num_rows = self.row_definitions.len() + 1;
            num_columns = self.column_definitions.len() + 1;
        } else {
            // N cells
            num_rows = self.row_definitions.len();
            num_columns = self.column_definitions.len();
        }
        let extra_rows = row_range.end.saturating_sub(num_rows);
        let extra_columns = column_range.end.saturating_sub(num_columns);

        for _ in 0..extra_rows {
            self.row_definitions.push(TrackSize {
                min_size: self.implicit_row_size,
                max_size: self.implicit_row_size,
            });
        }

        for _ in 0..extra_columns {
            self.column_definitions.push(TrackSize {
                min_size: self.implicit_column_size,
                max_size: self.implicit_column_size,
            });
        }

        let pos = self.items.partition_point(|item| item.z_order <= z_order);
        self.items.insert(
            pos,
            GridItem {
                row_range,
                column_range,
                z_order,
                widget,
                line_alignment,
            },
        );

        self.invalidate_child_filter()
    }

    /// Inserts items with auto-flow placement.
    #[composable]
    pub fn insert<T: Insertable>(&mut self, items: T) {
        let row_len = match self.auto_flow_dir {
            FlowDirection::Row => self.column_count(),
            FlowDirection::Column => self.row_count(),
        };

        let mut row = self.auto_flow_row;
        let mut column = self.auto_flow_col;
        let flow = self.auto_flow_dir;

        let mut flow_cursor = FlowCursor {
            grid: self,
            row,
            column,
            row_len,
            flow,
        };

        if flow_cursor.flow == FlowDirection::Column {
            mem::swap(&mut flow_cursor.row, &mut flow_cursor.column);
        }

        items.insert(&mut flow_cursor);

        if flow_cursor.flow == FlowDirection::Column {
            mem::swap(&mut flow_cursor.row, &mut flow_cursor.column);
        }

        let row = flow_cursor.row;
        let column = flow_cursor.column;

        self.auto_flow_row = row;
        self.auto_flow_col = column;
    }

    /// Invalidates the cached child widget filter.
    fn invalidate_child_filter(&self) {
        self.cached_child_filter.set(None);
    }

    fn items_in_track(&self, axis: TrackAxis, index: usize) -> impl Iterator<Item = &GridItem> {
        self.items.iter().filter(move |item| item.is_in_track(axis, index))
    }

    /// Computes the sizes of rows or columns.
    ///
    /// * `available_space`: max size across track direction (columns => max width, rows => max height).
    /// * `column_sizes`: contains the result of `compute_track_sizes` on the columns when sizing the rows. Used as an additional constraint for rows that size to content.
    fn compute_track_sizes(
        &self,
        layout_ctx: &mut LayoutCtx,
        env: &Environment,
        axis: TrackAxis,
        available_space: f64,
        row_gap: f64,
        column_gap: f64,
        column_layout: Option<&[GridTrackLayout]>,
    ) -> ComputeTrackSizeResult {
        let tracks = match axis {
            TrackAxis::Row => &self.row_definitions[..],
            TrackAxis::Column => &self.column_definitions[..],
        };

        let gap = match axis {
            TrackAxis::Row => row_gap,
            TrackAxis::Column => column_gap,
        };

        let num_tracks = tracks.len();
        let num_gutters = if num_tracks > 1 { num_tracks - 1 } else { 0 };

        let mut base_size = vec![0.0; num_tracks];
        let mut growth_limit = vec![0.0; num_tracks];
        let mut baselines = vec![None; num_tracks];

        // for each track, update base_size and growth limit
        for i in 0..num_tracks {
            // If automatic sizing is requested (for min or max), compute the items natural sizes (result of layout with unbounded boxconstraints)
            // Also, for rows (axis == TrackAxis::Row) with AlignItems::Baseline, compute the max baseline offset of all items in the track
            let mut natural_sizes = Vec::new();
            if tracks[i].min_size == GridLength::Auto || tracks[i].max_size == GridLength::Auto {
                for item in self.items_in_track(axis, i) {
                    // if we already have a column layout, constrain available space by the size of the column range
                    let constraints = if let Some(column_layout) = column_layout {
                        // width of the column range, including gutters
                        let w = track_span_width(column_layout, item.column_range.clone(), column_gap);
                        BoxConstraints::new(0.0..w, ..)
                    } else {
                        BoxConstraints::new(.., ..)
                    };
                    // FIXME: nothing prevents the widget to return an infinite size
                    // Q: is it the responsibility of the widget to handle unbounded constraints?
                    let natural_size = item.widget.speculative_layout(layout_ctx, constraints, env);
                    natural_sizes.push(natural_size);
                }
            }

            let max_natural_baseline: Option<f64> = natural_sizes.iter().filter_map(|m| m.baseline).reduce(f64::max);
            baselines[i] = max_natural_baseline;

            // adjust sizes for baseline alignment
            if let Some(max_natural_baseline) = max_natural_baseline {
                if axis == TrackAxis::Row && self.align_items == AlignItems::Baseline {
                    for nat_size in natural_sizes.iter_mut() {
                        nat_size.size.height += max_natural_baseline - nat_size.baseline.unwrap_or(0.0);
                    }
                }
            }

            let max_natural_size = natural_sizes
                .iter()
                .map(|m| axis.width(m.size))
                .reduce(f64::max)
                .unwrap_or(0.0);

            // apply min size constraint
            match tracks[i].min_size {
                GridLength::Fixed(min) => {
                    base_size[i] = min.to_dips(layout_ctx.scale_factor, available_space);
                }
                GridLength::Auto => {
                    base_size[i] = max_natural_size;
                }
                GridLength::Flex(_) => {}
            };

            // apply max size constraint
            match tracks[i].max_size {
                GridLength::Fixed(max) => {
                    growth_limit[i] = max.to_dips(layout_ctx.scale_factor, available_space);
                }
                GridLength::Auto => {
                    // same as min size constraint
                    growth_limit[i] = max_natural_size;
                }
                GridLength::Flex(_) => growth_limit[i] = f64::INFINITY,
            };

            if growth_limit[i] < base_size[i] {
                growth_limit[i] = base_size[i];
            }
        }

        // Maximize non-flex tracks, on the "free space", which is the available space minus
        // the space already taken by the fixed- and auto-sized element, and the gutter gaps.
        let mut free_space = available_space - base_size.iter().sum::<f64>() - (num_gutters as f64) * gap;
        for i in 0..tracks.len() {
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
            for t in tracks {
                if let GridLength::Flex(x) = t.max_size {
                    flex_total += x
                }
            }
            for i in 0..num_tracks {
                if let GridLength::Flex(x) = tracks[i].max_size {
                    let fr = x / flex_total;
                    base_size[i] = base_size[i].max(fr * free_space);
                }
            }
        }

        //tracing::trace!("{:?} base_size={:?}, growth_limit={:?}", axis, base_size, growth_limit);

        // grid line positions
        let mut layout = Vec::with_capacity(num_tracks);
        let mut pos = 0.0;
        for i in 0..base_size.len() {
            let size = base_size[i];
            let baseline = baselines[i];
            layout.push(GridTrackLayout { pos, size, baseline });
            pos += size;
            if i != base_size.len() - 1 {
                pos += gap;
            }
        }

        ComputeTrackSizeResult { layout, size: pos }
    }

    /*/// Returns the calculated column layout.
    pub fn get_column_layout(&self) -> (f64, Vec<GridTrackLayout>) {
        let grid_layout = self.calculated_layout.get();
        (grid_layout.width, grid_layout.column_layout)
    }

    pub fn get_row_layout(&self) -> (f64, Vec<GridTrackLayout>) {
        let grid_layout = self.calculated_layout.get();
        (grid_layout.height, grid_layout.row_layout)
    }*/
}
