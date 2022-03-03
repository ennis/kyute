use crate::{
    align_boxes, bloom::Bloom, drawing::ToSkia, widget::prelude::*, Color, EnvKey, InternalEvent, Length, WidgetId,
};
use kyute::WidgetFilter;
use std::{
    cell::{Cell, RefCell},
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    sync::Arc,
};

pub const SHOW_GRID_LAYOUT_LINES: EnvKey<bool> = EnvKey::new("kyute.show_grid_layout_lines");

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GridLength {
    /// Size to content.
    Auto,
    /// Fixed size.
    Fixed(f64),
    /// Proportion of remaining space.
    Flex(f64),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum JustifyItems {
    Start,
    End,
    Center,
    Stretch,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

impl From<Length> for GridLength {
    fn from(length: Length) -> Self {
        match length {
            Length::Dip(dips) => GridLength::Fixed(dips),
            _ => {
                todo!("GridLength from Inches & Px")
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TrackSize {
    min_size: GridLength,
    max_size: GridLength,
}

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

pub trait GridSpan {
    fn resolve(&self, len: usize) -> Range<usize>;
}

impl GridSpan for usize {
    fn resolve(&self, _len: usize) -> Range<usize> {
        *self..*self + 1
    }
}

impl GridSpan for Range<usize> {
    fn resolve(&self, _len: usize) -> Range<usize> {
        self.clone()
    }
}

impl GridSpan for RangeInclusive<usize> {
    fn resolve(&self, _len: usize) -> Range<usize> {
        *self.start()..*self.end() + 1
    }
}

impl GridSpan for RangeFrom<usize> {
    fn resolve(&self, len: usize) -> Range<usize> {
        self.start..len
    }
}

impl GridSpan for RangeTo<usize> {
    fn resolve(&self, _len: usize) -> Range<usize> {
        0..self.end
    }
}

impl GridSpan for RangeToInclusive<usize> {
    fn resolve(&self, _len: usize) -> Range<usize> {
        0..(self.end + 1)
    }
}

impl GridSpan for RangeFull {
    fn resolve(&self, len: usize) -> Range<usize> {
        0..len
    }
}

#[derive(Clone, Debug)]
struct GridTrackLayout {
    pos: f64,
    size: f64,
    baseline: f64,
}

struct ComputeTrackSizeResult {
    layout: Vec<GridTrackLayout>,
    size: f64,
}

#[derive(Clone, Debug)]
pub struct Grid {
    id: WidgetId,
    /// Column sizes.
    columns: Vec<TrackSize>,
    /// Row sizes.
    rows: Vec<TrackSize>,
    /// List of grid items: widgets positioned inside the grid.
    items: Vec<GridItem>,

    /// Row template.
    row_template: GridLength,
    column_template: GridLength,
    row_gap: Length,
    column_gap: Length,

    align_items: AlignItems,
    justify_items: JustifyItems,

    row_layout: RefCell<Vec<GridTrackLayout>>,
    column_layout: RefCell<Vec<GridTrackLayout>>,

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
        Grid {
            id: WidgetId::here(),
            columns: vec![],
            rows: vec![],
            items: vec![],
            row_template: GridLength::Auto,
            column_template: GridLength::Auto,
            row_gap: Length::Dip(0.0),
            column_gap: Length::Dip(0.0),
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            row_layout: RefCell::new(vec![]),
            column_layout: RefCell::new(vec![]),
            cached_child_filter: Cell::new(None),
        }
    }

    /// Creates a single-column grid.
    pub fn column(width: impl Into<GridLength>) -> Grid {
        Self::with_columns([width.into()])
    }

    /// Creates a single-row grid.
    pub fn row(height: impl Into<GridLength>) -> Grid {
        Self::with_rows([height.into()])
    }

    /// Returns the current number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Returns the current number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn with_columns(columns: impl IntoIterator<Item = GridLength>) -> Grid {
        Self::with_rows_columns([], columns)
    }

    pub fn with_rows(rows: impl IntoIterator<Item = GridLength>) -> Grid {
        Self::with_rows_columns(rows, [])
    }

    pub fn with_rows_columns(
        rows: impl IntoIterator<Item = GridLength>,
        columns: impl IntoIterator<Item = GridLength>,
    ) -> Grid {
        Grid {
            id: WidgetId::here(),
            columns: columns
                .into_iter()
                .map(|size| TrackSize {
                    min_size: size,
                    max_size: size,
                })
                .collect(),
            rows: rows
                .into_iter()
                .map(|size| TrackSize {
                    min_size: size,
                    max_size: size,
                })
                .collect(),
            items: vec![],
            row_template: GridLength::Auto,
            column_template: GridLength::Auto,
            row_gap: Length::Dip(0.0),
            column_gap: Length::Dip(0.0),
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            row_layout: RefCell::new(vec![]),
            column_layout: RefCell::new(vec![]),
            cached_child_filter: Cell::new(None),
        }
    }

    pub fn row_gap(mut self, gap: impl Into<Length>) -> Self {
        self.row_gap = gap.into();
        self
    }

    pub fn column_gap(mut self, gap: impl Into<Length>) -> Self {
        self.column_gap = gap.into();
        self
    }

    pub fn row_template(mut self, size: GridLength) -> Self {
        self.row_template = size;
        self
    }

    pub fn column_template(mut self, size: GridLength) -> Self {
        self.column_template = size;
        self
    }

    pub fn align_items(mut self, align_items: AlignItems) -> Self {
        self.align_items = align_items;
        self
    }

    pub fn justify_items(mut self, justify_items: JustifyItems) -> Self {
        self.justify_items = justify_items;
        self
    }

    #[composable]
    pub fn with(mut self, row_span: impl GridSpan, column_span: impl GridSpan, widget: impl Widget + 'static) -> Self {
        self.add(row_span, column_span, widget);
        self
    }

    #[composable]
    pub fn add(&mut self, row_span: impl GridSpan, column_span: impl GridSpan, widget: impl Widget + 'static) {
        let row_range = row_span.resolve(self.rows.len());
        let column_range = column_span.resolve(self.columns.len());

        // add rows/columns as required
        let num_rows = self.rows.len();
        let num_columns = self.columns.len();
        let extra_rows = row_range.end.checked_sub(num_rows).unwrap_or(0);
        let extra_columns = column_range.end.checked_sub(num_columns).unwrap_or(0);
        for _ in 0..extra_rows {
            self.rows.push(TrackSize {
                min_size: self.row_template,
                max_size: self.row_template,
            });
        }
        for _ in 0..extra_columns {
            self.columns.push(TrackSize {
                min_size: self.column_template,
                max_size: self.column_template,
            });
        }

        self.items.push(GridItem {
            row_range,
            column_range,
            widget: Arc::new(WidgetPod::new(widget)),
        });

        self.invalidate_child_filter()
    }

    #[composable]
    pub fn add_row(&mut self, widget: impl Widget + 'static) {
        self.add(self.rows.len(), .., widget);
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
        column_layout: Option<&[GridTrackLayout]>,
    ) -> ComputeTrackSizeResult {
        let tracks = match axis {
            TrackAxis::Row => &self.rows[..],
            TrackAxis::Column => &self.columns[..],
        };

        let gap = match axis {
            TrackAxis::Row => self.row_gap.to_dips(layout_ctx.scale_factor),
            TrackAxis::Column => self.column_gap.to_dips(layout_ctx.scale_factor),
        };

        let num_tracks = tracks.len();
        let num_gutters = if num_tracks > 1 { num_tracks - 1 } else { 0 };

        let mut base_size = vec![0.0; num_tracks];
        let mut growth_limit = vec![0.0; num_tracks];
        let mut baselines = vec![0.0; num_tracks];

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
                        let column_gap = self.column_gap.to_dips(layout_ctx.scale_factor);
                        let w = track_span_width(column_layout, item.column_range.clone(), column_gap);
                        BoxConstraints::new(0.0..w, ..)
                    } else {
                        BoxConstraints::new(.., ..)
                    };
                    let natural_size = item.widget.layout(layout_ctx, constraints, env);
                    natural_sizes.push(natural_size);
                }
            }

            let max_natural_baseline: f64 = natural_sizes
                .iter()
                .filter_map(|m| m.baseline)
                .reduce(f64::max)
                .unwrap_or(0.0);
            baselines[i] = max_natural_baseline;

            // adjust sizes for baseline alignment
            if axis == TrackAxis::Row && self.align_items == AlignItems::Baseline {
                for nat_size in natural_sizes.iter_mut() {
                    nat_size.bounds.size.height += max_natural_baseline - nat_size.baseline.unwrap_or(0.0);
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
                    base_size[i] = min;
                }
                GridLength::Auto => {
                    base_size[i] = max_natural_size;
                }
                GridLength::Flex(_) => {}
            };

            // apply max size constraint
            match tracks[i].max_size {
                GridLength::Fixed(max) => {
                    growth_limit[i] = max;
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
        // the space already taken by the fixed- and auto-sized element, and the gutter gaps
        let mut free_space = available_space - base_size.iter().sum::<f64>() - (num_gutters as f64) * gap;
        for i in 0..tracks.len() {
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

        // distribute remaining spaces to flex tracks
        let mut flex_total = 0.0;
        for t in tracks {
            match t.max_size {
                GridLength::Flex(x) => flex_total += x,
                _ => {}
            }
        }
        for i in 0..num_tracks {
            match tracks[i].max_size {
                GridLength::Flex(x) => {
                    let fr = x / flex_total;
                    base_size[i] = base_size[i].max(fr * free_space);
                }
                _ => {}
            }
        }

        /*tracing::trace!(
            "{:?} base_size={:?}, growth_limit={:?}",
            axis,
            base_size,
            growth_limit
        );*/

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
                for item in self.items.iter() {
                    item.widget.event(ctx, event, env);
                }
            }
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // first measure the width of the columns
        let ComputeTrackSizeResult {
            layout: column_layout,
            size: width,
        } = self.compute_track_sizes(ctx, env, TrackAxis::Column, constraints.max_width(), None);
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
            Some(&column_layout[..]),
        );

        // layout items
        for item in self.items.iter() {
            let column_gap = self.column_gap.to_dips(ctx.scale_factor);
            let row_gap = self.row_gap.to_dips(ctx.scale_factor);

            let w: f64 = track_span_width(&column_layout, item.column_range.clone(), column_gap);
            let h: f64 = track_span_width(&row_layout, item.row_range.clone(), row_gap);

            let constraints = BoxConstraints::loose(Size::new(w, h));
            let item_measure = item.widget.layout(ctx, constraints, env);

            let mut x = column_layout[item.column_range.start].pos;
            let mut y = row_layout[item.row_range.start].pos;
            let baseline = row_layout[item.row_range.start].baseline;

            // position item inside the cell according to alignment mode
            y += match self.align_items {
                AlignItems::Start => 0.0,
                AlignItems::End => h - item_measure.size().height,
                AlignItems::Center => 0.5 * (h - item_measure.size().height),
                AlignItems::Stretch => {
                    // TODO handle stretch by modifying constraints
                    0.0
                }
                AlignItems::Baseline => {
                    // align to max baseline
                    // NOTE: we assume that the baseline doesn't vary between the minimal measurements
                    // obtained during row layout and the measurement with the final constraints.
                    baseline - item_measure.baseline.unwrap_or(item_measure.height())
                }
            };

            item.widget.set_child_offset(Offset::new(x, y));
        }

        self.row_layout.replace(row_layout);
        self.column_layout.replace(column_layout);
        Measurements::new(Rect::new(Point::origin(), Size::new(width, height)))
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        use skia_safe as sk;
        let height = bounds.size.height;
        let width = bounds.size.width;

        // draw debug grid lines
        if env.get(SHOW_GRID_LAYOUT_LINES).unwrap_or_default() {
            let row_layout = self.row_layout.borrow();
            let column_layout = self.column_layout.borrow();
            let paint = sk::Paint::new(Color::new(1.0, 0.5, 0.2, 1.0).to_skia(), None);

            for x in column_layout.iter().map(|x| x.pos).chain(std::iter::once(width)) {
                ctx.canvas.draw_line(
                    Point::new(x + 0.5, 0.5).to_skia(),
                    Point::new(x + 0.5, height + 0.5).to_skia(),
                    &paint,
                );
            }

            for y in row_layout.iter().map(|x| x.pos).chain(std::iter::once(height)) {
                ctx.canvas.draw_line(
                    Point::new(0.5, y + 0.5).to_skia(),
                    Point::new(width + 0.5, y + 0.5).to_skia(),
                    &paint,
                );
            }
        }

        // draw grid items
        for item in self.items.iter() {
            item.widget.paint(ctx, bounds, env);
        }
    }
}
