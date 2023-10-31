//! Grid layout.
//!
use crate::{
    composable,
    debug_util::DebugWriter,
    drawing::{Paint, ToSkia},
    element::TransformNode,
    layout::place_into,
    widget::Axis,
    Alignment, AnyWidget, ChangeFlags, Color, Element, ElementId, Environment, Event, EventCtx, Geometry,
    HitTestResult, LayoutCtx, LayoutParams, LengthOrPercentage, PaintCtx, RouteEventCtx, TreeCtx, Widget,
};
use kurbo::{Insets, Point, Size, Vec2};
use kyute2_macros::grid_template;
use skia_safe as sk;
use std::{any::Any, borrow::Cow, mem, ops::Range};
use tracing::{error, trace, trace_span};

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
}

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

impl GridAxis {
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
}

/// Returns the size of a box along the specified axis.
fn size_across(axis: GridAxis, size: Size) -> f64 {
    // TODO depends on the writing mode
    match axis {
        GridAxis::Row => size.height,
        GridAxis::Column => size.width,
    }
}

//grid_template! { GRID:[START] 100px 1fr 1fr [END] / [TOP] auto[BOTTOM] }

struct GridItem {
    area: GridArea,
    x_align: Alignment,
    y_align: Alignment,
    content: Box<dyn AnyWidget>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// WIDGET

/// A widget that layouts its content on a grid.
pub struct Grid {
    flow: FlowDirection,
    content: Vec<GridItem>,
    template: GridTemplate,
    row_gap: f64,
    column_gap: f64,
    row_background: Paint,
    alternate_row_background: Paint,
    row_gap_background: Paint,
    column_gap_background: Paint,
}

impl Grid {
    /// Creates a new grid widget from the specified template.
    ///
    /// See `grid_template!`.
    pub fn from_template(template: &GridTemplate) -> Grid {
        Grid {
            flow: FlowDirection::Column,
            content: vec![],
            template: template.clone(),
            row_gap: 0.0,
            column_gap: 0.0,
            row_background: Default::default(),
            alternate_row_background: Default::default(),
            row_gap_background: Default::default(),
            column_gap_background: Default::default(),
        }
    }

    /// Inserts an element into the grid.
    pub fn add(&mut self, area: GridArea, x_align: Alignment, y_align: Alignment, content: impl AnyWidget + 'static) {
        self.content.push(GridItem {
            area,
            x_align,
            y_align,
            content: Box::new(content),
        })
    }
}

impl Widget for Grid {
    type Element = GridElement;

    fn build(self, cx: &mut TreeCtx, id: ElementId) -> Self::Element {
        let content: Vec<_> = self
            .content
            .into_iter()
            .enumerate()
            .map(|(i, item)| GridItemElement {
                // FIXME: ID shouldn't be derived from index
                content: TransformNode::new(cx.build_with_id(&i, item.content)),
                area: item.area,
                x_align: Default::default(),
                row_range: Default::default(),
                column_range: Default::default(),
                y_align: Default::default(),
                natural_baseline: 0.0,
            })
            .collect();

        GridElement {
            id,
            flow: self.flow,
            content,
            template: self.template,
            column_layout: vec![],
            row_layout: vec![],
            row_baselines: vec![],
            column_gap: self.column_gap,
            row_gap: self.row_gap,
            row_background: self.row_background,
            alternate_row_background: self.alternate_row_background,
            row_gap_background: self.row_gap_background,
            column_gap_background: self.column_gap_background,
        }
    }

    fn update(self, cx: &mut TreeCtx, element: &mut Self::Element) -> ChangeFlags {
        let mut change_flags = ChangeFlags::empty();

        if self.template != element.template
            || self.row_gap != element.row_gap
            || self.column_gap != element.column_gap
            || self.flow != element.flow
        {
            element.template = self.template;
            element.row_gap = self.row_gap;
            element.column_gap = self.column_gap;
            element.flow = self.flow;
            change_flags |= ChangeFlags::GEOMETRY | ChangeFlags::PAINT;
        }

        {
            // we can't compare paints for now, so assume they change
            // TODO: compare paints and don't repaint if they haven't changed
            element.row_background = self.row_background;
            element.alternate_row_background = self.alternate_row_background;
            element.row_gap_background = self.row_gap_background;
            element.column_gap_background = self.column_gap_background;
            change_flags |= ChangeFlags::PAINT;
        }

        let num_items = self.content.len();
        let num_items_in_element = element.content.len();
        for (i, item) in self.content.into_iter().enumerate() {
            // TODO: match by item identity
            if i < num_items_in_element {
                change_flags |= cx.update_with_id(&i, item.content, &mut element.content[i].content.content);
            } else {
                let elem = GridItemElement {
                    content: TransformNode::new(cx.build(item.content)),
                    area: item.area,
                    x_align: item.x_align,
                    y_align: item.y_align,
                    row_range: Default::default(),
                    column_range: Default::default(),
                    natural_baseline: 0.0,
                };
                element.content.push(elem);
            }
        }

        element.content.truncate(num_items);
        change_flags

        /*reconcile_elements(
            cx,
            self.content,
            &mut element.content,
            env,
            |w| &w.content,
            |item| &mut item.content,
            |cx, item, env| GridItemElement {
                content: TransformNode::new(item.content.build(cx, env)),
                area: item.area,
                x_align: item.x_align,
                y_align: item.y_align,
                row_range: Default::default(),
                column_range: Default::default(),
                natural_baseline: 0.0,
            },
            |cx, item, item_elem, env| {
                let mut change_flags = ChangeFlags::empty();
                change_flags |= Widget::update(item.content, cx, &mut item_elem.content.content, env);

                if item.area != item_elem.area || item.x_align != item_elem.x_align || item.y_align != item_elem.y_align
                {
                    item_elem.area = item.area;
                    item_elem.x_align = item.x_align;
                    item_elem.y_align = item.y_align;
                    change_flags |= ChangeFlags::GEOMETRY;
                }

                change_flags
            },
        );*/
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// ELEMENT

/// Grid layout element.
pub struct GridElement {
    id: ElementId,
    flow: FlowDirection,
    /// Child elements.
    content: Vec<GridItemElement>,
    /// Grid positions of child elements (same size as `self.content`).
    template: GridTemplate,

    // computed values
    column_layout: Vec<GridTrackLayout>,
    row_layout: Vec<GridTrackLayout>,
    row_baselines: Vec<f64>,

    column_gap: f64,
    row_gap: f64,
    row_background: Paint,
    alternate_row_background: Paint,
    row_gap_background: Paint,
    column_gap_background: Paint,
}

impl Element for GridElement {
    fn id(&self) -> ElementId {
        self.id
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, params: &LayoutParams) -> Geometry {
        let _span = trace_span!("grid layout", id = ?self.id).entered();

        // TODO the actual direction of rows and columns depends on the writing mode
        // When (or if) we support other writing modes, rewrite this. Layout is complicated!

        // first, place items in the grid (i.e. resolve their grid areas into "definite areas")
        let (row_count, column_count) = self.place_items();

        // first measure the width of the columns
        let (width, _width_changed) = self.compute_track_sizes(
            ctx,
            params,
            GridAxis::Column,
            column_count,
            params.max.width,
            self.row_gap,
            self.column_gap,
        );

        // then measure the height of the rows, which may depend on the width of the columns
        // Note: it may go the other way around (width of columns that depend on the height of the rows)
        // but we choose to do it like this
        let (height, _height_changed) = self.compute_track_sizes(
            ctx,
            params,
            GridAxis::Row,
            row_count,
            params.max.height,
            self.row_gap,
            self.column_gap,
        );

        //trace!("final row layout {:?}", row_layout);
        //trace!("final column layout {:?}", column_layout);

        // Maximum baselines for each row of the grid (y-offset to the row's starting y-coordinate)
        let mut row_baselines: Vec<f64> = vec![0.0; self.row_layout.len()];

        {
            let _span = trace_span!("grid collect row baselines").entered();
            for item in self.content.iter() {
                if item.y_align == Alignment::FirstBaseline || item.y_align == Alignment::LastBaseline {
                    // TODO last baseline
                    let row = item.row_range.start as usize;
                    row_baselines[row] = row_baselines[row].max(item.natural_baseline);
                }
            }
        }

        {
            let _span = trace_span!("grid item measure & place").entered();
            for item in self.content.iter_mut() {
                //let (column_start, column_end) = item.column_range;
                //let (row_start, row_end) = item.row_range.get();
                let w: f64 = track_span_width(&self.column_layout, item.column_range.clone(), self.column_gap);
                let h: f64 = track_span_width(&self.row_layout, item.row_range.clone(), self.row_gap);

                debug_assert!(
                    item.column_range.start < self.column_layout.len() as u32
                        && item.column_range.end <= self.column_layout.len() as u32
                        && item.row_range.start < self.row_layout.len() as u32
                        && item.row_range.end <= self.row_layout.len() as u32
                );

                let mut subconstraints = *params;
                subconstraints.max.width = w;
                subconstraints.max.height = h;
                subconstraints.min.width = 0.0;
                subconstraints.min.height = 0.0;

                let child_layout = item.content.layout(ctx, &subconstraints);
                trace!("[{:?}] constraints: {:?}", item.content.id(), subconstraints);
                trace!("[{:?}] layout: {:?}", item.content.id(), child_layout);

                // place the item within its grid cell
                let row = item.row_range.start as usize;
                let column = item.column_range.start as usize;
                let cell_pos = Vec2::new(self.column_layout[column].pos, self.row_layout[row].pos);

                let content_pos = place_into(
                    child_layout.size,
                    child_layout.baseline,
                    Size::new(w, h),
                    Some(row_baselines[row]),
                    item.x_align,
                    item.y_align,
                    &Insets::ZERO,
                );

                // TODO round to pixel
                let offset = (cell_pos + content_pos).round();
                item.content.set_offset(offset);
                //child_layouts.push((Size::new(w, h), child_layout));
            }
        }

        // TODO baseline
        Geometry::new(Size::new(width, height))
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &mut Event) -> ChangeFlags {
        // We don't care about events, but propagate if necessary
        if let Some(target) = event.next_target() {
            let child = self
                .content
                .iter_mut()
                .find(|e| e.content.id() == target)
                .expect("invalid child specified");
            child.content.event(ctx, event)
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

    fn natural_size(&mut self, axis: Axis, params: &LayoutParams) -> f64 {
        // Not sure how to implement that more efficiently other than just recomputing the whole layout
        todo!()
    }

    fn natural_baseline(&mut self, params: &LayoutParams) -> f64 {
        // argh
        todo!()
    }

    fn hit_test(&self, ctx: &mut HitTestResult, position: Point) -> bool {
        let mut hit = false;
        for item in self.content.iter() {
            hit |= item.content.hit_test(ctx, position);
        }
        trace!("grid hit test: {}", hit);
        hit
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        for item in self.content.iter_mut() {
            item.content.paint(ctx)
        }

        let width = self.column_layout.last().map(|x| x.pos + x.size).unwrap_or(0.0);
        let height = self.row_layout.last().map(|x| x.pos + x.size).unwrap_or(0.0);

        // draw debug grid lines
        let mut surface = ctx.surface.surface();
        let canvas = surface.canvas();
        let paint = sk::Paint::new(Color::new(1.0, 0.5, 0.2, 1.0).to_skia(), None);
        for x in self
            .column_layout
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
            .row_layout
            .iter()
            .map(|x| x.pos)
            .chain(std::iter::once(height - 1.0))
        {
            canvas.draw_line(
                Point::new(0.5, y + 0.5).to_skia(),
                Point::new(width + 0.5, y + 0.5).to_skia(),
                &paint,
            );
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn debug(&self, visitor: &mut DebugWriter) {
        visitor.type_name("GridElement");
        visitor.property("flow", self.flow);
        //visitor.property("template", self.template);
        visitor.property("row_gap", self.row_gap);
        visitor.property("column_gap", self.column_gap);
        //visitor.property("row_background", self.row_background);
        //visitor.property("alternate_row_background", self.alternate_row_background);
        //visitor.property("row_gap_background", self.row_gap_background);
        //visitor.property("column_gap_background", self.column_gap_background);
        for item in self.content.iter() {
            visitor.child("item", &item.content);
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// INTERNALS

struct GridItemElement {
    area: GridArea,
    x_align: Alignment,
    y_align: Alignment,
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
    fn update_natural_baseline(&mut self, parent_constraints: &LayoutParams) {
        let mut constraints = *parent_constraints;
        constraints.min.width = 0.0;
        constraints.max.width = f64::INFINITY;
        constraints.min.height = 0.0;
        constraints.max.height = f64::INFINITY;
        self.natural_baseline = self.content.natural_baseline(&constraints);
    }

    /// Returns the natural width of this grid element.
    ///
    /// # Arguments
    /// * parent_constraints constraints passed to the GridElement's `layout` method
    fn get_natural_width(&mut self, parent_constraints: &LayoutParams) -> f64 {
        let mut constraints = *parent_constraints;
        constraints.min.width = 0.0;
        constraints.max.width = f64::INFINITY;
        constraints.min.height = 0.0;
        constraints.max.height = f64::INFINITY;
        self.content.natural_size(Axis::Horizontal, &constraints)
    }

    /// Returns the natural height of this grid element, possibly under constrained column widths.
    ///
    /// # Arguments
    /// * parent_constraints constraints passed to the GridElement's `layout` method
    /// * column_layout the result of column layout. Used to constrain the width of the item.
    /// * column_gap column gap of the parent grid
    fn get_natural_height(
        &mut self,
        parent_constraints: &LayoutParams,
        column_layout: &[GridTrackLayout],
        column_gap: f64,
    ) -> f64 {
        let mut constraints = *parent_constraints;
        constraints.min.width = 0.0;
        constraints.max.width = f64::INFINITY;
        constraints.min.height = 0.0;
        constraints.max.height = f64::INFINITY;

        // if we already determined the size of the columns,
        // constrain the width by the size of the column range
        let w = track_span_width(column_layout, self.column_range.clone(), column_gap);
        //trace!("using column width constraint: max_width = {}", w);
        constraints.max.width = w;

        self.content.natural_size(Axis::Vertical, &constraints)
    }
}

/// Position and size of a grid track.
#[derive(Clone, Debug, PartialEq, Default)]
struct GridTrackLayout {
    pos: f64,
    size: f64,
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

impl GridElement {
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
        trace!("=== [{:?}] placing {} items ===", self.id, self.content.len());
        trace!(
            "{} template rows, {} template columns, autoflow: {:?}",
            self.template.rows.len(),
            self.template.columns.len(),
            self.flow
        );

        let mut final_row_count = self.template.rows.len();
        let mut final_column_count = self.template.columns.len();

        let mut flow_cursor = FlowCursor {
            row: 0,
            column: 0,
            flow_dir_size: match self.flow {
                FlowDirection::Row => self.template.columns.len() as u32,
                FlowDirection::Column => self.template.rows.len() as u32,
            },
            flow: self.flow,
        };

        for item in self.content.iter_mut() {
            if item.area.is_null() {
                // this should not happen because we check for null areas when adding the item to
                // the grid, but check it here as well for good measure
                error!("null grid area during placement (id={:?})", item.content.id());
                continue;
            }

            let (row_range, column_range) = flow_cursor.place(item.area);
            final_row_count = final_row_count.max(row_range.end as usize);
            final_column_count = final_column_count.max(column_range.end as usize);

            trace!(
                "{:?}: rows {}..{} columns {}..{} (area = {:?}, cursor = {:?})",
                item.content.id(),
                row_range.start,
                row_range.end,
                column_range.start,
                column_range.end,
                item.area,
                flow_cursor
            );

            item.row_range = row_range.start..row_range.end;
            item.column_range = column_range.start..column_range.end;
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
    ///
    /// # Return value
    ///
    /// A tuple `(total_size, changed)` with the total track size including gaps + whether the grid line positions have changed since last time.
    fn compute_track_sizes(
        &mut self,
        layout_ctx: &mut LayoutCtx,
        constraints: &LayoutParams,
        axis: GridAxis,
        track_count: usize,
        available_space: f64,
        row_gap: f64,
        column_gap: f64,
    ) -> (f64, bool) {
        let _span = trace_span!("grid track sizing", ?axis).entered();

        let gap = match axis {
            GridAxis::Row => row_gap,
            GridAxis::Column => column_gap,
        };

        trace!("=== [{:?}] laying out: {:?} ===", self.id, axis);

        // base sizes (cross-axis) of the tracks (column widths, or row heights)
        let mut base_size = vec![0.0; track_count];
        let mut growth_limit = vec![0.0; track_count];
        let num_gutters = if track_count > 1 { track_count - 1 } else { 0 };

        // for each track, update base_size and growth limit
        for i in 0..track_count {
            trace!("--- laying out track {} ---", i);

            // If automatic sizing is requested (for min or max), compute the items natural sizes (result of layout with unbounded boxconstraints)
            // Also, for rows (axis == TrackAxis::Row) with AlignItems::Baseline, compute the max baseline offset of all items in the track
            let track_size = self.template.track_size(axis, i);
            let auto_sized = track_size.min_size == TrackBreadth::Auto || track_size.max_size == TrackBreadth::Auto;
            let mut max_natural_size = 0.0f64;

            if auto_sized {
                match axis {
                    GridAxis::Column => {
                        for item in items_in_track_mut(&mut self.content, axis, i) {
                            let width = item.get_natural_width(constraints);
                            max_natural_size = max_natural_size.max(width);
                        }
                    }
                    GridAxis::Row => {
                        // first pass: update and calculate max baseline
                        let mut max_baseline = 0.0f64;
                        for item in items_in_track_mut(&mut self.content, axis, i) {
                            item.update_natural_baseline(constraints);
                            max_baseline = max_baseline.max(item.natural_baseline);
                        }

                        // 2nd pass: calculate max height
                        for item in items_in_track_mut(&mut self.content, axis, i) {
                            let mut height = item.get_natural_height(constraints, &self.column_layout, column_gap);
                            if item.y_align == Alignment::FirstBaseline || item.y_align == Alignment::LastBaseline {
                                // adjust the returned size with additional padding to account for baseline alignment
                                height += max_baseline - item.natural_baseline;
                            }
                            max_natural_size = max_natural_size.max(height);
                        }
                    }
                }

                trace!("track #{} max_natural_size={:?}", i, max_natural_size);
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
                let track_size = self.template.track_size(axis, i);
                if let TrackBreadth::Flex(x) = track_size.max_size {
                    flex_total += x
                }
            }
            for i in 0..(track_count as usize) {
                let track_size = self.template.track_size(axis, i);
                if let TrackBreadth::Flex(x) = track_size.max_size {
                    let fr = x / flex_total;
                    base_size[i] = base_size[i].max(fr * free_space);
                }
            }
        }

        //tracing::trace!("{:?} base_size={:?}, growth_limit={:?}", axis, base_size, growth_limit);
        let mut layout = match axis {
            GridAxis::Row => &mut self.row_layout,
            GridAxis::Column => &mut self.column_layout,
        };

        // update grid line positions
        let mut changed = false;
        layout.resize(track_count, Default::default());
        let mut pos = 0.0;
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

        (pos, changed)
    }
}
