use crate::{drawing::ToSkia, widget::prelude::*, Color, EnvKey, Length};
use std::{
    cell::RefCell,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
    sync::Arc,
};

pub const SHOW_GRID_LAYOUT_LINES: EnvKey<bool> = EnvKey::new("kyute.show_grid_layout_lines");

#[derive(Copy, Clone, Debug)]
pub enum GridLength {
    /// Size to content.
    Auto,
    /// Fixed size.
    Fixed(f64),
    /// Proportion of remaining space.
    Flex(f64),
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
#[derive(Copy, Clone, Debug)]
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
}

#[derive(Clone, Debug)]
pub struct Grid {
    /// Column sizes.
    columns: Vec<TrackSize>,
    /// Row sizes.
    rows: Vec<TrackSize>,
    /// List of grid items: widgets positioned inside the grid.
    items: Vec<GridItem>,

    /// Row template.
    row_template: GridLength,
    column_template: GridLength,

    row_layout: RefCell<Vec<GridTrackLayout>>,
    column_layout: RefCell<Vec<GridTrackLayout>>,
}

impl Grid {
    pub fn new() -> Grid {
        Grid {
            columns: vec![],
            rows: vec![],
            items: vec![],
            row_template: GridLength::Auto,
            column_template: GridLength::Auto,
            row_layout: RefCell::new(vec![]),
            column_layout: RefCell::new(vec![]),
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
            row_layout: RefCell::new(vec![]),
            column_layout: RefCell::new(vec![]),
        }
    }

    pub fn row_template(mut self, size: GridLength) -> Self {
        self.row_template = size;
        self
    }

    pub fn column_template(mut self, size: GridLength) -> Self {
        self.column_template = size;
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
    ) -> (Vec<GridTrackLayout>, f64) {
        let tracks = match axis {
            TrackAxis::Row => &self.rows[..],
            TrackAxis::Column => &self.columns[..],
        };

        let num_tracks = tracks.len();

        let mut base_size = vec![0.0; num_tracks];
        let mut growth_limit = vec![0.0; num_tracks];

        for i in 0..num_tracks {
            match tracks[i].min_size {
                GridLength::Fixed(x) => {
                    base_size[i] = x;
                }
                GridLength::Auto => {
                    for item in self.items_in_track(axis, i) {
                        let size = item
                            .widget
                            .layout(layout_ctx, BoxConstraints::tight(Size::zero()), env)
                            .size();
                        base_size[i] = base_size[i].max(axis.width(size));
                    }
                }
                _ => {}
            };

            match tracks[i].max_size {
                GridLength::Fixed(x) => {
                    growth_limit[i] = x;
                }
                GridLength::Auto => {
                    for item in self.items_in_track(axis, i) {
                        let constraints = if let Some(column_layout) = column_layout {
                            let w: f64 = column_layout[item.column_range.clone()].iter().map(|x| x.size).sum();
                            BoxConstraints::new(0.0..w, ..)
                        } else {
                            BoxConstraints::new(.., ..)
                        };
                        let size = item.widget.layout(layout_ctx, constraints, env).size();
                        growth_limit[i] = growth_limit[i].max(axis.width(size));
                    }
                }
                GridLength::Flex(_) => growth_limit[i] = f64::INFINITY,
            };

            if growth_limit[i] < base_size[i] {
                growth_limit[i] = base_size[i];
            }
        }

        // Maximize non-flex tracks
        let mut free_space = available_space - base_size.iter().sum::<f64>();
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
        for &size in base_size.iter() {
            layout.push(GridTrackLayout { pos, size });
            pos += size;
        }
        (layout, pos)
    }
}

impl Widget for Grid {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        for item in self.items.iter() {
            item.widget.event(ctx, event, env);
        }
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        // first measure the width of the columns
        let (column_layout, width) =
            self.compute_track_sizes(ctx, env, TrackAxis::Column, constraints.max_width(), None);
        // then measure the height of the rows, which may depend on the width of the columns
        // Note: it may go the other way around (width of columns that depend on the height of the rows)
        // but we choose to do it like this
        let (row_layout, height) = self.compute_track_sizes(
            ctx,
            env,
            TrackAxis::Row,
            constraints.max_height(),
            Some(&column_layout[..]),
        );

        // layout items
        for item in self.items.iter() {
            let w: f64 = column_layout[item.column_range.clone()].iter().map(|x| x.size).sum();
            let h: f64 = row_layout[item.row_range.clone()].iter().map(|x| x.size).sum();

            let constraints = BoxConstraints::loose(Size::new(w, h));
            item.widget.layout(ctx, constraints, env);

            let x = column_layout[item.column_range.start].pos;
            let y = row_layout[item.row_range.start].pos;
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
