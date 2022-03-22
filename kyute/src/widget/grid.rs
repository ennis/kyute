use crate::{
    bloom::Bloom,
    drawing::ToSkia,
    style::{BoxStyle, Paint, PaintCtxExt},
    widget::prelude::*,
    Color, Data, EnvKey, InternalEvent, Length, RoundToPixel, ValueRef, WidgetFilter, WidgetId,
};
use euclid::Size2D;
use std::{
    cell::{Cell, RefCell},
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    sync::Arc,
};

pub const SHOW_GRID_LAYOUT_LINES: EnvKey<bool> = EnvKey::new("kyute.show_grid_layout_lines");

/// Description of a grid track (row or column).
#[derive(Clone, Debug, PartialEq)]
pub struct GridTrackDefinition {
    /// Track length.
    min_size: GridLength,
    max_size: GridLength,
    /// Optional track name.
    pub name: Option<String>,
}

impl GridTrackDefinition {
    pub fn new(length: impl Into<GridLength>) -> GridTrackDefinition {
        let length = length.into();
        GridTrackDefinition {
            min_size: length,
            max_size: length,
            name: None,
        }
    }

    pub fn named(name: impl Into<String>, length: impl Into<GridLength>) -> GridTrackDefinition {
        let length = length.into();
        GridTrackDefinition {
            min_size: length,
            max_size: length,
            name: Some(name.into()),
        }
    }
}

impl From<GridLength> for GridTrackDefinition {
    fn from(length: GridLength) -> Self {
        GridTrackDefinition::new(length)
    }
}

/// Length of a grid track.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridLength {
    /// Size to content.
    Auto,
    /// Fixed size.
    Fixed(Length),
    /// Proportion of remaining space.
    Flex(f64),
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

/*impl From<Length> for GridLength {
    fn from(length: Length) -> Self {
        GridLength::Fixed(length)
        match length {
            Length::Dip(dips) => GridLength::Fixed(dips),
            _ => {
                todo!("GridLength from Inches & Px")
            }
        }
    }
}*/

/// Orientation of a grid track.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug)]
pub struct GridItem {
    row_range: Range<usize>,
    column_range: Range<usize>,
    widget: Arc<WidgetPod>,
}

impl GridItem {
    /*fn track_span(&self, axis: TrackAxis) -> Range<usize> {
        match axis {
            TrackAxis::Row => self.row_range.clone(),
            TrackAxis::Column => self.column_range.clone(),
        }
    }*/

    fn is_in_track(&self, axis: TrackAxis, index: usize) -> bool {
        match axis {
            TrackAxis::Row => self.row_range.start == index,
            TrackAxis::Column => self.column_range.start == index,
        }
    }
}

#[derive(Clone, Debug)]
struct GridTrackLayout {
    pos: f64,
    size: f64,
    baseline: Option<f64>,
}

struct ComputeTrackSizeResult {
    layout: Vec<GridTrackLayout>,
    size: f64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GridSpan<'a> {
    Single(usize),
    Range(Range<usize>),
    RangeTo(RangeTo<usize>),
    RangeFrom(RangeFrom<usize>),
    RangeInclusive(RangeInclusive<usize>),
    RangeToInclusive(RangeToInclusive<usize>),
    RangeFull,
    Named(&'a str),
}

impl<'a> GridSpan<'a> {
    fn resolve(&self, track_definitions: &[GridTrackDefinition]) -> Range<usize> {
        match self {
            GridSpan::Single(v) => *v..*v + 1,
            GridSpan::Range(v) => v.clone(),
            GridSpan::RangeTo(v) => 0..v.end,
            GridSpan::RangeFrom(v) => v.start..track_definitions.len(),
            GridSpan::RangeInclusive(v) => *v.start()..*v.end() + 1,
            GridSpan::RangeToInclusive(v) => 0..(v.end + 1),
            GridSpan::RangeFull => 0..track_definitions.len(),
            GridSpan::Named(name) => {
                let track = track_definitions
                    .iter()
                    .position(|t| t.name.as_deref().map(|n| n == *name).unwrap_or(false))
                    .expect("no such named track in grid");
                track..track + 1
            }
        }
    }
}

impl<'a> From<usize> for GridSpan<'a> {
    fn from(v: usize) -> Self {
        GridSpan::Single(v)
    }
}

impl<'a> From<Range<usize>> for GridSpan<'a> {
    fn from(v: Range<usize>) -> Self {
        GridSpan::Range(v)
    }
}

impl<'a> From<RangeTo<usize>> for GridSpan<'a> {
    fn from(v: RangeTo<usize>) -> Self {
        GridSpan::RangeTo(v)
    }
}

impl<'a> From<RangeFrom<usize>> for GridSpan<'a> {
    fn from(v: RangeFrom<usize>) -> Self {
        GridSpan::RangeFrom(v)
    }
}

impl<'a> From<RangeInclusive<usize>> for GridSpan<'a> {
    fn from(v: RangeInclusive<usize>) -> Self {
        GridSpan::RangeInclusive(v)
    }
}

impl<'a> From<RangeToInclusive<usize>> for GridSpan<'a> {
    fn from(v: RangeToInclusive<usize>) -> Self {
        GridSpan::RangeToInclusive(v)
    }
}

impl<'a> From<RangeFull> for GridSpan<'a> {
    fn from(_: RangeFull) -> Self {
        GridSpan::RangeFull
    }
}

impl<'a> From<&'a str> for GridSpan<'a> {
    fn from(v: &'a str) -> Self {
        GridSpan::Named(v)
    }
}

/// Item in a grid row.
pub struct GridRowItem<'a> {
    pub column: GridSpan<'a>,
    pub widget: Arc<WidgetPod>,
}

/// Represents a row of widgets to be inserted in a grid.
pub struct GridRow<'a> {
    items: Vec<GridRowItem<'a>>,
}

impl<'a> GridRow<'a> {
    /// Creates an empty `GridRow`.
    pub fn new() -> GridRow<'a> {
        GridRow { items: vec![] }
    }

    /// Adds an item to the row.
    pub fn add(&mut self, column: impl Into<GridSpan<'a>>, widget: impl Widget + 'static) {
        self.items.push(GridRowItem {
            column: column.into(),
            widget: Arc::new(WidgetPod::new(widget)),
        })
    }
}

impl<'a> Default for GridRow<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W> From<W> for GridRow<'static>
where
    W: Widget + 'static,
{
    fn from(widget: W) -> Self {
        let mut row = GridRow::new();
        row.add(.., widget);
        row
    }
}

#[derive(Clone, Debug)]
struct CachedGridLayout {
    constraints: BoxConstraints,
    measurements: Measurements,
    row_layout: Vec<GridTrackLayout>,
    column_layout: Vec<GridTrackLayout>,
    row_gap: f64,
    column_gap: f64,
}

// FIXME: cloning anything with a widget id in it is extremely suspect: widgets are only clone for caching,
// but using it in regular code to make multiple copies of a widget will break a lot of things, similar to forgetting #[composable]

#[derive(Clone, Debug)]
pub struct Grid {
    id: WidgetId,
    /// Column sizes.
    column_definitions: Vec<GridTrackDefinition>,
    /// Row sizes.
    row_definitions: Vec<GridTrackDefinition>,
    /// List of grid items: widgets positioned inside the grid.
    items: Vec<GridItem>,

    /// Row template.
    row_template: GridLength,
    column_template: GridLength,
    row_gap: Length,
    column_gap: Length,

    align_items: AlignItems,
    justify_items: JustifyItems,

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
    //row_layout: RefCell<Vec<GridTrackLayout>>,
    //column_layout: RefCell<Vec<GridTrackLayout>>,

    // FIXME this is ugly, there's probably the same problem with the child filter
    cached_layout: Arc<RefCell<Option<CachedGridLayout>>>,
    cached_child_filter: Cell<Option<Bloom<WidgetId>>>,
}

/// Returns the size of a column span
fn track_span_width(layout: &[GridTrackLayout], span: Range<usize>, gap: f64) -> f64 {
    layout[span.clone()].iter().map(|x| x.size).sum::<f64>() + gap * (span.len() as isize - 1).max(0) as f64
}

impl Grid {
    /// Invalidate the cached child widget filter.
    fn invalidate_child_filter(&self) {
        self.cached_child_filter.set(None);
    }

    /// Creates a new grid, initially without any row or column definitions.
    pub fn new() -> Grid {
        Grid::with_rows_columns([], [])
    }

    /// Creates a single-column grid.
    pub fn column(width: impl Into<GridTrackDefinition>) -> Grid {
        Self::with_column_definitions([width.into()])
    }

    /// Creates a single-row grid.
    pub fn row(height: impl Into<GridTrackDefinition>) -> Grid {
        Self::with_row_definitions([height.into()])
    }

    /// Returns the current number of rows
    pub fn row_count(&self) -> usize {
        self.row_definitions.len()
    }

    /// Returns the current number of columns
    pub fn column_count(&self) -> usize {
        self.column_definitions.len()
    }

    pub fn with_column_definitions(columns: impl IntoIterator<Item = GridTrackDefinition>) -> Grid {
        Self::with_rows_columns([], columns)
    }

    pub fn with_row_definitions(rows: impl IntoIterator<Item = GridTrackDefinition>) -> Grid {
        Self::with_rows_columns(rows, [])
    }

    /// Appends a new row to this grid.
    pub fn push_row_definition(&mut self, def: GridTrackDefinition) {
        self.row_definitions.push(def);
    }

    /// Appends a new column to this grid.
    pub fn push_column_definition(&mut self, def: GridTrackDefinition) {
        self.column_definitions.push(def);
    }

    /// Creates a new grid with the specified rows and columns.
    pub fn with_rows_columns(
        rows: impl IntoIterator<Item = GridTrackDefinition>,
        columns: impl IntoIterator<Item = GridTrackDefinition>,
    ) -> Grid {
        Grid {
            id: WidgetId::here(),
            column_definitions: columns.into_iter().collect(),
            row_definitions: rows.into_iter().collect(),
            items: vec![],
            row_template: GridLength::Auto,
            column_template: GridLength::Auto,
            row_gap: Length::Dip(0.0),
            column_gap: Length::Dip(0.0),
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            row_background: Default::default(),
            alternate_row_background: Default::default(),
            row_gap_background: Default::default(),
            column_gap_background: Default::default(),
            cached_layout: Arc::new(RefCell::new(None)),
            cached_child_filter: Cell::new(None),
        }
    }

    /// Sets the size of the gap between rows.
    pub fn row_gap(mut self, gap: impl Into<Length>) -> Self {
        self.row_gap = gap.into();
        self
    }

    /// Sets the size of the gap between rows.
    pub fn set_row_gap(&mut self, gap: impl Into<Length>) {
        self.row_gap = gap.into();
    }

    /// Sets the size of the gap between columns.
    pub fn column_gap(mut self, gap: impl Into<Length>) -> Self {
        self.column_gap = gap.into();
        self
    }

    /// Sets the size of the gap between columns.
    pub fn set_column_gap(&mut self, gap: impl Into<Length>) {
        self.column_gap = gap.into();
    }

    /// Sets the template for implicit row definitions.
    pub fn row_template(mut self, size: GridLength) -> Self {
        self.row_template = size;
        self
    }

    /// Sets the template for implicit row definitions.
    pub fn set_row_template(&mut self, size: GridLength) {
        self.row_template = size;
    }

    /// Sets the template for implicit column definitions.
    pub fn column_template(mut self, size: GridLength) -> Self {
        self.column_template = size;
        self
    }

    /// Sets the template for implicit row definitions.
    pub fn set_column_template(&mut self, size: GridLength) {
        self.column_template = size;
    }

    pub fn align_items(mut self, align_items: AlignItems) -> Self {
        self.align_items = align_items;
        self
    }

    pub fn justify_items(mut self, justify_items: JustifyItems) -> Self {
        self.justify_items = justify_items;
        self
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

    // TODO remove? rename to `item()`
    #[composable]
    pub fn with_item<'a>(
        mut self,
        row_span: impl Into<GridSpan<'a>>,
        column_span: impl Into<GridSpan<'a>>,
        widget: impl Widget + 'static,
    ) -> Self {
        self.add_item(row_span, column_span, widget);
        self
    }

    #[composable]
    pub fn add_item<'a>(
        &mut self,
        row_span: impl Into<GridSpan<'a>>,
        column_span: impl Into<GridSpan<'a>>,
        widget: impl Widget + 'static,
    ) {
        let widget = Arc::new(WidgetPod::new(widget));
        self.push_item_inner(row_span, column_span, widget);
    }

    #[composable]
    pub fn add_item_pod<'a>(
        &mut self,
        row_span: impl Into<GridSpan<'a>>,
        column_span: impl Into<GridSpan<'a>>,
        widget: Arc<WidgetPod>,
    ) {
        self.push_item_inner(row_span, column_span, widget);
    }

    /// Resolves the specified column span to column indices.
    pub fn resolve_column_span(&self, column_span: GridSpan) -> Range<usize> {
        column_span.resolve(&self.column_definitions)
    }

    /// Resolves the specified row span to row indices.
    pub fn resolve_row_span(&self, row_span: GridSpan) -> Range<usize> {
        row_span.resolve(&self.row_definitions)
    }

    #[composable]
    fn push_item_inner<'a>(
        &mut self,
        row_span: impl Into<GridSpan<'a>>,
        column_span: impl Into<GridSpan<'a>>,
        widget: Arc<WidgetPod>,
    ) {
        let row_range = row_span.into().resolve(&self.row_definitions);
        let column_range = column_span.into().resolve(&self.column_definitions);

        // add rows/columns as required
        let num_rows = self.row_definitions.len();
        let num_columns = self.column_definitions.len();
        let extra_rows = row_range.end.saturating_sub(num_rows);
        let extra_columns = column_range.end.saturating_sub(num_columns);
        for _ in 0..extra_rows {
            self.row_definitions.push(GridTrackDefinition {
                min_size: self.row_template,
                max_size: self.row_template,
                name: None,
            });
        }
        for _ in 0..extra_columns {
            self.column_definitions.push(GridTrackDefinition {
                min_size: self.column_template,
                max_size: self.column_template,
                name: None,
            });
        }

        self.items.push(GridItem {
            row_range,
            column_range,
            widget,
        });

        self.invalidate_child_filter()
    }

    #[composable]
    pub fn add_row<'a>(&mut self, row: impl Into<GridRow<'a>>) {
        let row = row.into();
        let row_index = self.row_definitions.len();
        for item in row.items {
            self.push_item_inner(row_index, item.column, item.widget)
        }
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
                    let natural_size = item.widget.layout(layout_ctx, constraints, env);
                    natural_sizes.push(natural_size);
                }
            }

            let max_natural_baseline: Option<f64> = natural_sizes.iter().filter_map(|m| m.baseline).reduce(f64::max);
            baselines[i] = max_natural_baseline;

            // adjust sizes for baseline alignment
            if let Some(max_natural_baseline) = max_natural_baseline {
                if axis == TrackAxis::Row && self.align_items == AlignItems::Baseline {
                    for nat_size in natural_sizes.iter_mut() {
                        nat_size.bounds.size.height += max_natural_baseline - nat_size.baseline.unwrap_or(0.0);
                    }
                }
            }

            let max_natural_size = natural_sizes
                .iter()
                .map(|m| axis.width(m.size()))
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
}

impl Default for Grid {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Grid {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Internal(InternalEvent::UpdateChildFilter { filter }) => {
                // intercept the UpdateChildFilter event to return the cached filter instead
                // of recalculating it
                if let Some(ref cached_filter) = self.cached_child_filter.get() {
                    filter.extend(cached_filter);
                } else {
                    let mut child_filter = WidgetFilter::new();
                    for item in self.items.iter() {
                        let mut e = Event::Internal(InternalEvent::UpdateChildFilter {
                            filter: &mut child_filter,
                        });
                        item.widget.event(ctx, &mut e, env);
                    }
                    self.cached_child_filter.set(Some(child_filter));
                    filter.extend(&child_filter);
                }
            }
            event => {
                // run the events through the items in reverse order
                // in order to give priority to items inserted last. This is important given
                // that grid items can overlap, and we have no concept of Z-order
                // FIXME: add Z-order for overlapping widgets
                for item in self.items.iter().rev() {
                    item.widget.event(ctx, event, env);
                }
            }
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // try to use the cached layout first
        {
            let cached_layout = (&*self.cached_layout).borrow();
            if let Some(cached_layout) = cached_layout.as_ref() {
                if cached_layout.constraints == constraints {
                    return cached_layout.measurements;
                }
            }
        }

        // compute gap sizes
        let column_gap = self
            .column_gap
            .to_dips(ctx.scale_factor, constraints.finite_max_width().unwrap_or(0.0));
        let row_gap = self
            .row_gap
            .to_dips(ctx.scale_factor, constraints.finite_max_height().unwrap_or(0.0));

        // first measure the width of the columns
        let ComputeTrackSizeResult {
            layout: column_layout,
            size: width,
        } = self.compute_track_sizes(
            ctx,
            env,
            TrackAxis::Column,
            constraints.max_width(),
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
            env,
            TrackAxis::Row,
            constraints.max_height(),
            row_gap,
            column_gap,
            Some(&column_layout[..]),
        );

        // layout items
        for item in self.items.iter() {
            let w: f64 = track_span_width(&column_layout, item.column_range.clone(), column_gap);
            let h: f64 = track_span_width(&row_layout, item.row_range.clone(), row_gap);

            let constraints = BoxConstraints::loose(Size::new(w, h));
            let item_measure = item.widget.layout(ctx, constraints, env);

            //eprintln!("item_measure({})={:?}", item.widget.widget().debug_name(), item_measure);

            let mut x = column_layout[item.column_range.start].pos;
            let mut y = row_layout[item.row_range.start].pos;
            let row_baseline = row_layout[item.row_range.start].baseline;
            //eprintln!("row baseline={:?}", row_baseline);

            // position item inside the cell according to alignment mode
            match self.align_items {
                AlignItems::End => y += h - item_measure.size().height,
                AlignItems::Center => y += 0.5 * (h - item_measure.size().height),
                AlignItems::Baseline => {
                    if let Some(baseline) = item_measure.baseline {
                        // NOTE: normally if any item in the row has a baseline, then the row itself
                        // should have a baseline as well (row_baseline shouldn't be empty)
                        if let Some(row_baseline) = row_baseline {
                            // NOTE: we assume that the baseline doesn't vary between the minimal measurements
                            // obtained during row layout and the measurement with the final constraints.
                            y += row_baseline - baseline;
                        }
                    }
                }
                _ => {}
            };

            // position item inside the cell according to alignment mode
            match self.justify_items {
                JustifyItems::End => x += w - item_measure.size().width,
                JustifyItems::Center => x += 0.5 * (w - item_measure.size().width),
                _ => {}
            };

            /*eprintln!(
                "item offset({})={:?}",
                item.widget.widget().debug_name(),
                Offset::new(x, y)
            );*/
            item.widget
                .set_child_offset(Offset::new(x, y).round_to_pixel(ctx.scale_factor));
        }

        let measurements = Measurements::new(Rect::new(Point::origin(), Size::new(width, height)));
        self.cached_layout.replace(Some(CachedGridLayout {
            constraints,
            measurements,
            row_layout,
            column_layout,
            row_gap,
            column_gap,
        }));
        measurements
    }

    fn paint(&self, ctx: &mut PaintCtx, env: &Environment) {
        use skia_safe as sk;
        let height = ctx.bounds.size.height;
        let width = ctx.bounds.size.width;

        let layout = (&*self.cached_layout).borrow();
        let layout = layout.as_ref().expect("grid layout not calculated before paint");
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
                    &BoxStyle::new().fill(bg),
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
                    &BoxStyle::new().fill(self.row_gap_background.clone()),
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
                    &BoxStyle::new().fill(self.column_gap_background.clone()),
                );
            }
        }

        // draw debug grid lines
        if env.get(SHOW_GRID_LAYOUT_LINES).unwrap_or_default() {
            let paint = sk::Paint::new(Color::new(1.0, 0.5, 0.2, 1.0).to_skia(), None);
            for x in column_layout.iter().map(|x| x.pos).chain(std::iter::once(width - 1.0)) {
                ctx.canvas.draw_line(
                    Point::new(x + 0.5, 0.5).to_skia(),
                    Point::new(x + 0.5, height + 0.5).to_skia(),
                    &paint,
                );
            }
            for y in row_layout.iter().map(|x| x.pos).chain(std::iter::once(height - 1.0)) {
                ctx.canvas.draw_line(
                    Point::new(0.5, y + 0.5).to_skia(),
                    Point::new(width + 0.5, y + 0.5).to_skia(),
                    &paint,
                );
            }
        }

        // draw grid items
        for item in self.items.iter() {
            item.widget.paint(ctx, env);
        }
    }
}
