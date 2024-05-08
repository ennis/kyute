//! Grid layout.
//!
use std::{any::Any, borrow::Cow, mem, ops::Range};

use kurbo::{Insets, Point, Rect, Size, Vec2};
use skia_safe as skia;
use tracing::{error, trace};
use tracy_client::span;

use crate::{
    debug_util::DebugWriter, drawing::ToSkia, element::TransformNode, layout::place_into, Alignment, AnyWidget,
    BoxConstraints, ChangeFlags, Color, Element, ElementId, Event, EventCtx, Geometry, HitTestResult, LayoutCtx,
    LengthOrPercentage, PaintCtx, TreeCtx, Widget,
};

/// Grid renderers.
pub trait GridStyle: PartialEq {
    fn insets(&self) -> Insets;
    fn draw(&self, ctx: &mut PaintCtx, layout: &GridLayout);
}

/// Default grid style, draws nothing.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DefaultGridStyle;

/// Length of a grid track.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TrackBreadth {
    /// Size to content.
    Auto,
    /// Fixed size.
    Fixed(f64),
    /// Proportion of remaining space.
    Flex(f64),
}

impl Default for TrackBreadth {
    fn default() -> Self {
        TrackBreadth::Auto
    }
}

impl From<f64> for TrackBreadth {
    fn from(length: f64) -> Self {
        TrackBreadth::Fixed(length)
    }
}

/*
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum JustifyItems {
    Start,
    End,
    Center,
    // TODO currently ignored
    Stretch,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum AlignItems {
    Start,
    End,
    Center,
    // TODO currently ignored
    Stretch,
    Baseline,
}*/

/// Grid flow behavior.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum FlowDirection {
    /// Fill rows first
    Row,
    /// Fill columns first
    Column,
}

/// Sizing behavior of a grid track.
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct TrackSize {
    pub min_size: TrackBreadth,
    pub max_size: TrackBreadth,
}

impl TrackSize {
    /// Defines a track that is sized according to the provided TrackBreadth value.
    pub const fn new(size: TrackBreadth) -> TrackSize {
        TrackSize {
            min_size: size,
            max_size: size,
        }
    }

    /// Defines a track that is sized according to the provided TrackBreadth value.
    pub const fn fixed(size: f64) -> TrackSize {
        TrackSize::new(TrackBreadth::Fixed(size))
    }

    ///
    pub const fn flex(factor: f64) -> TrackSize {
        TrackSize::new(TrackBreadth::Flex(factor))
    }

    /// Defines minimum and maximum sizes for the track.
    pub const fn minmax(min_size: TrackBreadth, max_size: TrackBreadth) -> TrackSize {
        TrackSize { min_size, max_size }
    }

    pub const fn auto() -> TrackSize {
        TrackSize {
            min_size: TrackBreadth::Auto,
            max_size: TrackBreadth::Auto,
        }
    }
}

impl From<LengthOrPercentage> for TrackSize {
    fn from(value: LengthOrPercentage) -> Self {
        match value {
            LengthOrPercentage::Px(length) => TrackSize::new(TrackBreadth::Fixed(length)),
            LengthOrPercentage::Percentage(percentage) => TrackSize::new(TrackBreadth::Flex(percentage)),
        }
    }
}

impl From<f64> for TrackSize {
    fn from(value: f64) -> Self {
        TrackSize::new(TrackBreadth::Fixed(value))
    }
}

/// Defines a grid's rows and columns.
#[derive(Clone, Debug, PartialEq)]
pub struct GridTemplate {
    pub rows: Cow<'static, [TrackSize]>,
    pub columns: Cow<'static, [TrackSize]>,
    pub auto_rows: TrackSize,
    pub auto_columns: TrackSize,
}

impl GridTemplate {
    fn track_size(&self, axis: GridAxis, index: usize) -> TrackSize {
        match axis {
            GridAxis::Row => self.rows.get(index).cloned().unwrap_or(self.auto_rows),
            GridAxis::Column => self.columns.get(index).cloned().unwrap_or(self.auto_columns),
        }
    }
}

impl Default for GridTemplate {
    fn default() -> Self {
        GridTemplate {
            rows: Cow::Borrowed(&[]),
            columns: Cow::Borrowed(&[]),
            auto_rows: Default::default(),
            auto_columns: Default::default(),
        }
    }
}

/// Vertical grid line index.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ColumnLineIndex(pub u32);

/// Horizontal grid line index.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RowLineIndex(pub u32);

/// Describes the position of a grid item.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GridArea {
    /// Start row. If None, the item is positioned using autoflow.
    pub row: Option<u32>,
    /// Start column. If None, the item is positioned using autoflow.
    pub column: Option<u32>,
    /// Rows spanned by the item.
    pub row_span: u32,
    /// Columns spanned by the item.
    pub column_span: u32,
}

impl Default for GridArea {
    /// Returns the default `GridArea`, which is a 1x1 cell positioned using autoflow.
    fn default() -> Self {
        GridArea {
            row: None,
            column: None,
            row_span: 1,
            column_span: 1,
        }
    }
}

impl GridArea {
    pub fn is_null(&self) -> bool {
        self.row_span == 0 || self.column_span == 0
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Orientation of a grid track.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum GridAxis {
    /// Row (or _inline_) axis (follows the text).
    Row,
    /// Column (or _block_) axis, perpendicular to the flow of text.
    Column,
}

/*impl GridAxis {
    fn visual_axis(self) -> Axis {
        match self {
            GridAxis::Row => Axis::Horizontal,
            GridAxis::Column => Axis::Vertical,
        }
    }

    fn cross_axis(self) -> Axis {
        match self {
            GridAxis::Row => Axis::Vertical,
            GridAxis::Column => Axis::Horizontal,
        }
    }
}*/

/*
/// Returns the size of a box along the specified axis.
fn size_across(axis: GridAxis, size: Size) -> f64 {
    // TODO depends on the writing mode
    match axis {
        GridAxis::Row => size.height,
        GridAxis::Column => size.width,
    }
}*/

//grid_template! { GRID:[START] 100px 1fr 1fr [END] / [TOP] auto[BOTTOM] }

// TODO this should be shared across widgets
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct GridItemAlignment {
    pub x_align: Alignment,
    pub y_align: Alignment,
}

impl GridItemAlignment {
    pub const CENTER: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::CENTER,
        y_align: Alignment::CENTER,
    };
    pub const TOP_LEFT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::START,
        y_align: Alignment::START,
    };
    pub const TOP_CENTER: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::CENTER,
        y_align: Alignment::START,
    };
    pub const TOP_RIGHT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::END,
        y_align: Alignment::START,
    };
    pub const CENTER_LEFT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::START,
        y_align: Alignment::CENTER,
    };
    pub const CENTER_RIGHT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::END,
        y_align: Alignment::CENTER,
    };
    pub const BOTTOM_LEFT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::START,
        y_align: Alignment::END,
    };
    pub const BOTTOM_CENTER: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::CENTER,
        y_align: Alignment::END,
    };
    pub const BOTTOM_RIGHT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::END,
        y_align: Alignment::END,
    };
    pub const BASELINE_LEFT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::START,
        y_align: Alignment::FirstBaseline,
    };
    pub const BASELINE_CENTER: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::CENTER,
        y_align: Alignment::FirstBaseline,
    };
    pub const BASELINE_RIGHT: GridItemAlignment = GridItemAlignment {
        x_align: Alignment::END,
        y_align: Alignment::FirstBaseline,
    };

    pub const fn new(x_align: Alignment, y_align: Alignment) -> GridItemAlignment {
        GridItemAlignment { x_align, y_align }
    }
}

pub struct GridItem {
    pub area: GridArea,
    pub alignment: GridItemAlignment,
    pub content: WidgetPtr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// WIDGET

#[derive(Clone, Debug, PartialEq)]
pub struct GridOptions {
    pub flow: FlowDirection,
    pub rows: Cow<'static, [TrackSize]>,
    pub columns: Cow<'static, [TrackSize]>,
    pub auto_rows: TrackSize,
    pub auto_columns: TrackSize,
    pub row_gap: f64,
    pub column_gap: f64,
    //pub template: GridTemplate,
}

impl Default for GridOptions {
    fn default() -> Self {
        GridOptions {
            flow: FlowDirection::Column,
            rows: Default::default(),
            columns: Default::default(),
            auto_rows: Default::default(),
            auto_columns: Default::default(),
            row_gap: 0.0,
            column_gap: 0.0,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// ELEMENT

/// Computed layout of a grid.
#[derive(Default, Debug, Clone)]
pub struct GridLayout {
    pub geometry: Geometry,
    pub columns: Vec<GridTrackLayout>,
    pub rows: Vec<GridTrackLayout>,
    pub row_baselines: Vec<f64>,
}

impl GridLayout {
    /// Positions of the lines between rows.
    pub fn inner_row_lines(&self) -> impl Iterator<Item = f64> + '_ {
        self.rows.iter().skip(1).map(|x| x.pos)
    }

    /// Positions of the lines between columns.
    pub fn inner_column_lines(&self) -> impl Iterator<Item = f64> + '_ {
        self.columns.iter().skip(1).map(|x| x.pos)
    }
}

/// Grid layout element.
pub struct Grid<S> {
    style: S,
    /// Child elements.
    items: Vec<GridItemElement>,
    options: GridOptions,
    /// Computed layout
    layout: GridLayout,
}

impl<S: GridStyle + 'static> Widget for Grid<S> {
    fn layout(&mut self, ctx: &mut LayoutCtx, box_constraints: &BoxConstraints) -> Geometry {
        let _span = span!("Grid layout");

        // apply style insets
        let style_insets = self.style.insets();
        let box_constraints = box_constraints.deflate(style_insets);

        // TODO the actual direction of rows and columns depends on the writing mode
        // When (or if) we support other writing modes, rewrite this. Layout is complicated!

        // first, place items in the grid (i.e. resolve their grid areas into "definite areas")
        let (row_count, column_count) = self.place_items();

        // lay out the columns first
        let (width, _width_changed) = self.layout_tracks(
            ctx,
            &box_constraints,
            GridAxis::Column,
            column_count,
            box_constraints.max.width,
            self.options.row_gap,
            self.options.column_gap,
            (style_insets.x0, style_insets.x1),
        );

        // then lay out the rows, which may depend on the width of the columns
        // Note: it may go the other way around (width of columns that depend on the height of the rows)
        // but we choose to do it like this
        let (height, _height_changed) = self.layout_tracks(
            ctx,
            &box_constraints,
            GridAxis::Row,
            row_count,
            box_constraints.max.height,
            self.options.row_gap,
            self.options.column_gap,
            (style_insets.y0, style_insets.y1),
        );

        //trace!("final row layout {:?}", row_layout);
        //trace!("final column layout {:?}", column_layout);

        // Maximum baselines for each row of the grid (y-offset to the row's starting y-coordinate)
        let mut row_baselines: Vec<f64> = vec![0.0; self.layout.rows.len()];

        {
            let _span = span!("grid layout: collect row baselines");
            for item in self.items.iter() {
                if item.alignment.y_align == Alignment::FirstBaseline
                    || item.alignment.y_align == Alignment::LastBaseline
                {
                    // TODO last baseline
                    let row = item.row_range.start as usize;
                    row_baselines[row] = row_baselines[row].max(item.natural_baseline);
                }
            }
        }

        {
            let _span = span!("grid layout: item measure & place");
            for item in self.items.iter_mut() {
                //let (column_start, column_end) = item.column_range;
                //let (row_start, row_end) = item.row_range.get();
                let w: f64 = track_span_width(&self.layout.columns, item.column_range.clone(), self.options.column_gap);
                let h: f64 = track_span_width(&self.layout.rows, item.row_range.clone(), self.options.row_gap);

                debug_assert!(
                    item.column_range.start < self.layout.columns.len() as u32
                        && item.column_range.end <= self.layout.columns.len() as u32
                        && item.row_range.start < self.layout.rows.len() as u32
                        && item.row_range.end <= self.layout.rows.len() as u32
                );

                let child_layout = ctx.layout(
                    &mut item.content,
                    &BoxConstraints {
                        min: Size::ZERO,
                        max: Size::new(w, h),
                    },
                );
                //trace!("[{:?}] constraints: {:?}", item.content.id(), subconstraints);
                //trace!("[{:?}] layout: {:?}", item.content.id(), child_layout);

                // place the item within its grid cell
                let row = item.row_range.start as usize;
                let column = item.column_range.start as usize;
                let cell_pos = Vec2::new(self.layout.columns[column].pos, self.layout.rows[row].pos);

                let content_pos = place_into(
                    child_layout.size,
                    child_layout.baseline,
                    Size::new(w, h),
                    Some(row_baselines[row]),
                    item.alignment.x_align,
                    item.alignment.y_align,
                    &Insets::ZERO,
                );

                // TODO round to pixel
                let offset = (cell_pos + content_pos).round();
                item.content.set_offset(offset);
                //child_layouts.push((Size::new(w, h), child_layout));
            }
        }

        let size = Size::new(width, height);
        self.layout.geometry = Geometry {
            size,
            baseline: if !row_baselines.is_empty() {
                Some(row_baselines[0] + style_insets.y0)
            } else {
                None
            },
            // FIXME: those should be propagated from the grid items
            bounding_rect: size.to_rect(),
            paint_bounding_rect: size.to_rect(),
        };
        self.layout.geometry
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        // We don't care about events, but propagate if necessary
        if let Some(target) = event.next_target() {
            let child = self
                .items
                .iter_mut()
                .find(|e| e.content.id() == target)
                .expect("invalid child specified");
            ctx.event(&mut child.content, event)
        } else {
            ChangeFlags::NONE
        }
    }

    /*fn route_event(&mut self, ctx: &mut RouteEventCtx, event: &mut Event) -> ChangeFlags {
        if let Some(next_target) = event.next_target() {
        } else {
            self.event(&mut ctx.inner, event)
        }
    }*/

    fn natural_width(&mut self, _height: f64) -> f64 {
        // Not sure how to implement that more efficiently other than just recomputing the whole layout
        todo!()
    }

    fn natural_height(&mut self, _width: f64) -> f64 {
        todo!()
    }

    fn natural_baseline(&mut self, _params: &BoxConstraints) -> f64 {
        // argh
        todo!()
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let mut hit = false;
        for item in self.items.iter() {
            hit |= item.content.hit_test(ctx, position);
        }
        trace!("grid hit test: {}", hit);
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        self.style.draw(ctx, &self.layout);

        for item in self.items.iter_mut() {
            ctx.paint(&mut item.content);
        }

        //let width = self.layout.columns.last().map(|x| x.pos + x.size).unwrap_or(0.0);
        //let height = self.layout.rows.last().map(|x| x.pos + x.size).unwrap_or(0.0);

        /*// draw debug grid lines
        let mut surface = ctx.surface.surface();
        let canvas = surface.canvas();
        let paint = skia::Paint::new(Color::new(1.0, 0.5, 0.2, 1.0).to_skia(), None);
        for x in self
            .layout
            .columns
            .iter()
            .map(|x| x.pos)
            .chain(std::iter::once(width - 1.0))
        {
            canvas.draw_line(
                Point::new(x + 0.5, 0.5).to_skia(),
                Point::new(x + 0.5, height + 0.5).to_skia(),
                &paint,
            );
        }
        for y in self
            .layout
            .rows
            .iter()
            .map(|x| x.pos)
            .chain(std::iter::once(height - 1.0))
        {
            canvas.draw_line(
                Point::new(0.5, y + 0.5).to_skia(),
                Point::new(width + 0.5, y + 0.5).to_skia(),
                &paint,
            );
        }*/
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("GridElement");
        visitor.property("flow", self.options.flow);
        visitor.property("row_gap", self.options.row_gap);
        visitor.property("column_gap", self.options.column_gap);
        for (i, item) in self.items.iter().enumerate() {
            let s = visitor.alloc_str(&format!("item[{i}]"));
            visitor.child(s, &item.content);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// INTERNALS

struct GridItemElement {
    area: GridArea,
    alignment: GridItemAlignment,
    content: TransformNode<Box<dyn Element>>,

    // --- Computed ---
    /// Final row range after autoflow.
    row_range: Range<u32>,
    /// Final column range after autoflow.
    column_range: Range<u32>,
    /// Calculated natural baseline of the element.
    natural_baseline: f64,
}

impl GridItemElement {
    fn update_natural_baseline(&mut self, parent_constraints: &BoxConstraints) {
        let mut constraints = *parent_constraints;
        constraints.min.width = 0.0;
        constraints.max.width = f64::INFINITY;
        constraints.min.height = 0.0;
        constraints.max.height = f64::INFINITY;
        self.natural_baseline = self.content.natural_baseline(&constraints);
    }

    /// Returns the natural width of this grid element.
    fn get_natural_width(&mut self) -> f64 {
        self.content.natural_width(f64::INFINITY)
    }

    /// Returns the natural height of this grid element, possibly under constrained column widths.
    ///
    /// # Arguments
    /// * parent_constraints constraints passed to the GridElement's `layout` method
    /// * column_layout the result of column layout. Used to constrain the width of the item.
    /// * column_gap column gap of the parent grid
    fn get_natural_height(&mut self, column_layout: &[GridTrackLayout], column_gap: f64) -> f64 {
        // if we already determined the size of the columns,
        // constrain the width by the size of the column range
        let width = track_span_width(column_layout, self.column_range.clone(), column_gap);
        //trace!("using column width constraint: max_width = {}", w);
        self.content.natural_height(width)
    }
}

/// Position and size of a grid track.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct GridTrackLayout {
    /// Position of the track. Includes insets specified by the style.
    pub pos: f64,
    /// Size of the track.
    pub size: f64,
}

/// Returns the size of a column span
fn track_span_width(layout: &[GridTrackLayout], span: Range<u32>, gap: f64) -> f64 {
    layout[span.start as usize..span.end as usize]
        .iter()
        .map(|x| x.size)
        .sum::<f64>()
        + gap * (span.len() as isize - 1).max(0) as f64
}

/// Helper to place items within a grid with autoflow.
#[derive(Debug)]
struct FlowCursor {
    row: u32,
    column: u32,
    flow_dir_size: u32,
    flow: FlowDirection,
}

impl FlowCursor {
    /// Advances the cursor to the specified column, possibly going to the next row if necessary.
    fn align(&mut self, column: u32) {
        if self.column < column {
            self.column = column;
        } else if self.column > column {
            self.row += 1;
            self.column = column;
        }
    }

    /// Advances the cursor by the specified row/column span.
    fn next(&mut self, row_span: u32, column_span: u32) -> (u32, u32) {
        let (row, column) = (self.row, self.column);
        self.column += column_span;
        if self.column >= self.flow_dir_size {
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

    /// Returns row range/column range
    fn place(&mut self, area: GridArea) -> (Range<u32>, Range<u32>) {
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

/*fn natural_constraints(
    parent: &LayoutParams,
    column_layout: Option<&[GridTrackLayout]>,
    column_range: Range<u32>,
    column_gap: f64,
) -> LayoutParams {
    let mut constraints = *parent;
    constraints.min.width = 0.0;
    constraints.max.width = f64::INFINITY;
    constraints.min.height = 0.0;
    constraints.max.height = f64::INFINITY;

    if let Some(column_layout) = column_layout {
        // if we already determined the size of the columns,
        // constrain the width by the size of the column range
        let w = track_span_width(column_layout, column_range, column_gap);
        trace!("using column width constraint: max_width = {}", w);
        constraints.max.width = w;
    }
    constraints
}*/

fn items_in_track_mut(
    content: &mut [GridItemElement],
    axis: GridAxis,
    index: usize,
) -> impl Iterator<Item = &mut GridItemElement> {
    content.iter_mut().filter(move |item| {
        // "grid line" items (those with row_range.len() == 0 or column_range.len() == 0)
        // are not considered to belong to any track, and don't intervene during track sizing
        if item.row_range.is_empty() || item.column_range.is_empty() {
            return false;
        }
        match axis {
            GridAxis::Row => item.row_range.start == index as u32,
            GridAxis::Column => item.column_range.start == index as u32,
        }
    })
}

impl<S> Grid<S> {
    /*fn items_in_track_mut(&mut self, axis: GridAxis, index: u32) -> impl Iterator<Item = &mut GridItemElement> {
        self.content.iter_mut().filter(move |item| {
            // "grid line" items (those with row_range.len() == 0 or column_range.len() == 0)
            // are not considered to belong to any track, and don't intervene during track sizing
            if item.row_range.is_empty() || item.column_range.is_empty() {
                return false;
            }
            match axis {
                GridAxis::Row => item.row_range.start == index,
                GridAxis::Column => item.column_range.start == index,
            }
        })
    }*/

    /// Computes the final row and column ranges of items inside the grid, resolving items placed with autoflow.
    ///
    /// Returns the computed row and column count.
    fn place_items(&mut self) -> (usize, usize) {
        let _span = span!("grid layout: place items");

        /*trace!("=== [{:?}] placing {} items ===", self.id, self.content.len());
        trace!(
            "{} template rows, {} template columns, autoflow: {:?}",
            self.template.rows.len(),
            self.template.columns.len(),
            self.flow
        );*/

        let mut final_row_count = self.options.rows.len();
        let mut final_column_count = self.options.columns.len();

        let mut flow_cursor = FlowCursor {
            row: 0,
            column: 0,
            flow_dir_size: match self.options.flow {
                FlowDirection::Row => self.options.columns.len() as u32,
                FlowDirection::Column => self.options.rows.len() as u32,
            },
            flow: self.options.flow,
        };

        for item in self.items.iter_mut() {
            if item.area.is_null() {
                // this should not happen because we check for null areas when adding the item to
                // the grid, but check it here as well for good measure
                error!("null grid area during placement (id={:?})", item.content.id());
                continue;
            }

            let (row_range, column_range) = flow_cursor.place(item.area);
            final_row_count = final_row_count.max(row_range.end as usize);
            final_column_count = final_column_count.max(column_range.end as usize);

            /*trace!(
                "{:?}: rows {}..{} columns {}..{} (area = {:?}, cursor = {:?})",
                item.content.id(),
                row_range.start,
                row_range.end,
                column_range.start,
                column_range.end,
                item.area,
                flow_cursor
            );*/

            item.row_range = row_range.start..row_range.end;
            item.column_range = column_range.start..column_range.end;
        }

        /*trace!(
            "final track count: rows={} columns={}",
            final_row_count,
            final_column_count
        );*/

        (final_row_count, final_column_count)
    }

    /// Computes the sizes of rows or columns.
    ///
    /// * `available_space`: max size across track direction (columns => max width, rows => max height).
    /// * `column_sizes`: contains the result of `compute_track_sizes` on the columns when sizing the rows. Used as an additional constraint for rows that size to content.
    /// * `insets`
    ///
    /// # Return value
    ///
    /// A tuple `(total_size, changed)` with the total track size including gaps + whether the grid line positions have changed since last time.
    /// **NOTE** the total size includes the initial inset offset.
    fn layout_tracks(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        constraints: &BoxConstraints,
        axis: GridAxis,
        track_count: usize,
        available_space: f64,
        row_gap: f64,
        column_gap: f64,
        insets: (f64, f64),
    ) -> (f64, bool) {
        let _span = span!("grid layout: grid track sizing");

        /// Helper function to return the size of a track.
        let get_track_size = |axis: GridAxis, index: usize| -> TrackSize {
            match axis {
                GridAxis::Row => self.options.rows.get(index).cloned().unwrap_or(self.options.auto_rows),
                GridAxis::Column => self
                    .options
                    .columns
                    .get(index)
                    .cloned()
                    .unwrap_or(self.options.auto_columns),
            }
        };

        let gap = match axis {
            GridAxis::Row => row_gap,
            GridAxis::Column => column_gap,
        };

        //trace!("=== [{:?}] laying out: {:?} ===", self.id, axis);
        let num_gutters = if track_count > 1 { track_count - 1 } else { 0 };

        // Base sizes (cross-axis) of the tracks (column widths, or row heights)
        let mut base_size = vec![0.0; track_count];
        // How big the tracks can grow (cross-axis).
        let mut growth_limit = vec![0.0; track_count];
        // In successive steps we'll update the base size and growth limit of each track.

        // First step: initialize base size & growth limit from min/max constraints and, for tracks
        // with auto sizing, from the natural size of the items in the track.
        for i in 0..track_count {
            //trace!("--- laying out track {} ---", i);

            let track_size = get_track_size(axis, i);

            // If automatic sizing is requested (for min or max), compute the maximum of the items'
            // natural sizes. This gives us the base size or growth limit of the track.
            //
            // Also, for rows (axis == TrackAxis::Row) compute the max baseline offset of all items in the track,
            // which is needed to determine the height of the row when aligning to the first or last baseline.
            let mut max_natural_size = 0.0f64;
            if track_size.min_size == TrackBreadth::Auto || track_size.max_size == TrackBreadth::Auto {
                match axis {
                    GridAxis::Column => {
                        for item in items_in_track_mut(&mut self.items, axis, i) {
                            let width = item.get_natural_width();
                            max_natural_size = max_natural_size.max(width);
                        }
                    }
                    GridAxis::Row => {
                        // first pass: update and calculate max baseline
                        let mut max_baseline = 0.0f64;
                        for item in items_in_track_mut(&mut self.items, axis, i) {
                            item.update_natural_baseline(constraints);
                            max_baseline = max_baseline.max(item.natural_baseline);
                        }

                        // 2nd pass: calculate max height
                        for item in items_in_track_mut(&mut self.items, axis, i) {
                            let mut height = item.get_natural_height(&self.layout.columns, column_gap);
                            if item.alignment.y_align == Alignment::FirstBaseline
                                || item.alignment.y_align == Alignment::LastBaseline
                            {
                                // adjust the returned size with additional padding to account for baseline alignment
                                height += max_baseline - item.natural_baseline;
                            }
                            max_natural_size = max_natural_size.max(height);
                        }
                    }
                }

                //trace!("track #{} max_natural_size={:?}", i, max_natural_size);
            }

            // apply min size constraint
            match track_size.min_size {
                TrackBreadth::Fixed(min) => {
                    base_size[i] = min;
                }
                TrackBreadth::Auto => {
                    base_size[i] = max_natural_size;
                }
                TrackBreadth::Flex(_) => {}
            };

            // apply max size constraint
            match track_size.max_size {
                TrackBreadth::Fixed(max) => {
                    growth_limit[i] = max;
                }
                TrackBreadth::Auto => {
                    // same as min size constraint
                    growth_limit[i] = max_natural_size;
                }
                TrackBreadth::Flex(_) => growth_limit[i] = f64::INFINITY,
            };

            // Sanitize base size & growth limit.
            // Not sure if this is necessary.
            if growth_limit[i] < base_size[i] {
                growth_limit[i] = base_size[i];
            }
        }

        // Step 2: maximize non-flex tracks, on the "free space", which is the available space minus
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

        // Step 3: distribute remaining space to flex tracks if the remaining free space is finite.
        // Otherwise they keep their assigned base sizes.
        if free_space.is_finite() {
            let mut flex_total = 0.0;
            for i in 0..track_count {
                let track_size = get_track_size(axis, i);
                if let TrackBreadth::Flex(x) = track_size.max_size {
                    flex_total += x
                }
            }
            for i in 0..(track_count as usize) {
                let track_size = get_track_size(axis, i);
                if let TrackBreadth::Flex(x) = track_size.max_size {
                    let fr = x / flex_total;
                    base_size[i] = base_size[i].max(fr * free_space);
                }
            }
        }

        //tracing::trace!("{:?} base_size={:?}, growth_limit={:?}", axis, base_size, growth_limit);
        let layout = match axis {
            GridAxis::Row => &mut self.layout.rows,
            GridAxis::Column => &mut self.layout.columns,
        };

        // update grid line positions
        let mut changed = false;
        layout.resize(track_count, Default::default());
        // start position is the initial inset offset given by the style
        let mut pos = insets.0;
        for i in 0..base_size.len() {
            let size = base_size[i];
            if layout[i].size != size || layout[i].pos != pos {
                changed = true;
                layout[i].size = size;
                layout[i].pos = pos;
            }

            pos += size;
            if i != base_size.len() - 1 {
                pos += gap;
            }
        }

        // don't forget to add the inset to the total size
        (pos + insets.1, changed)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// TABLE GRID STYLE

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TableGridStyle;

impl GridStyle for TableGridStyle {
    fn insets(&self) -> Insets {
        // FIXME hairline size?
        Insets::uniform(1.0)
    }

    fn draw(&self, ctx: &mut PaintCtx, layout: &GridLayout) {
        //let width = layout.columns.last().map(|x| x.pos + x.size).unwrap_or(0.0);
        //let height = layout.rows.last().map(|x| x.pos + x.size).unwrap_or(0.0);

        let size = layout.geometry.size;

        // Draw rect around the table
        ctx.with_canvas(|canvas| {
            let bg_cell = Color::from_hex("#363636");
            let bg_header = Color::from_hex("#4f4f4f");
            let border = Color::from_hex("#ababab");

            //let cell_paint = sk::Paint::new(bg_cell.to_skia(), None);

            let mut border_paint = skia::Paint::new(border.to_skia(), None);
            border_paint.set_style(skia::paint::Style::Stroke);
            let mut bg_paint = skia::Paint::new(bg_cell.to_skia(), None);
            bg_paint.set_style(skia::paint::Style::Fill);
            let mut header_paint = skia::Paint::new(bg_header.to_skia(), None);
            header_paint.set_style(skia::paint::Style::Fill);

            let rect = Rect {
                x0: 0.5,
                y0: 0.5,
                x1: size.width - 0.5,
                y1: size.height - 0.5,
            };
            canvas.draw_rect(rect.to_skia(), &bg_paint);
            canvas.draw_rect(rect.to_skia(), &border_paint);

            for y in layout.inner_row_lines() {
                canvas.draw_line(
                    Point::new(0.5, y + 0.5).to_skia(),
                    Point::new(size.width - 0.5, y + 0.5).to_skia(),
                    &border_paint,
                );
            }

            for x in layout.inner_column_lines() {
                canvas.draw_line(
                    Point::new(x + 0.5, 0.5).to_skia(),
                    Point::new(x + 0.5, size.height - 0.5).to_skia(),
                    &border_paint,
                );
            }
        });
    }
}
