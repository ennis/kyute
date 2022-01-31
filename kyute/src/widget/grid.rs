use crate::{
    composable, layout::Measurements, styling::Length, text::resolve_range, BoxConstraints, Data,
    Environment, Event, EventCtx, LayoutCtx, Offset, PaintCtx, Rect, Size, Widget, WidgetPod,
};
use kyute::Point;
use kyute_shell::drawing::{Color, ToSkia};
use std::{
    cell::RefCell,
    ops::{Range, RangeBounds, RangeFrom, RangeFull, RangeTo, RangeToInclusive},
};

#[derive(Copy, Clone, Debug)]
pub enum GridLength {
    Auto,
    Fixed(f64),
    Flex(f64),
}

#[derive(Copy, Clone, Debug)]
pub struct TrackSize {
    min_size: GridLength,
    max_size: GridLength,
}

#[derive(Copy, Clone, Debug)]
enum TrackKind {
    Row,
    Column,
}

#[derive(Clone, Debug)]
pub struct GridItem {
    row_range: Range<usize>,
    column_range: Range<usize>,
    widget: WidgetPod,
}

impl GridItem {
    fn track_span(&self, track_kind: TrackKind) -> Range<usize> {
        match track_kind {
            TrackKind::Row => self.row_range.clone(),
            TrackKind::Column => self.column_range.clone(),
        }
    }
}

fn track_width(track_kind: TrackKind, size: Size) -> f64 {
    match track_kind {
        TrackKind::Column => size.width,
        TrackKind::Row => size.height,
    }
}

pub trait GridSpan {
    fn resolve(&self, len: usize) -> Range<usize>;
}

impl GridSpan for usize {
    fn resolve(&self, len: usize) -> Range<usize> {
        *self..*self + 1
    }
}

impl GridSpan for Range<usize> {
    fn resolve(&self, len: usize) -> Range<usize> {
        self.clone()
    }
}

impl GridSpan for RangeFrom<usize> {
    fn resolve(&self, len: usize) -> Range<usize> {
        self.start..len
    }
}

impl GridSpan for RangeTo<usize> {
    fn resolve(&self, len: usize) -> Range<usize> {
        0..self.end
    }
}

impl GridSpan for RangeToInclusive<usize> {
    fn resolve(&self, len: usize) -> Range<usize> {
        0..(self.end + 1)
    }
}

impl GridSpan for RangeFull {
    fn resolve(&self, len: usize) -> Range<usize> {
        0..len
    }
}

#[derive(Clone, Debug)]
pub struct Grid {
    /// Column sizes.
    columns: Vec<TrackSize>,
    /// Row sizes.
    rows: Vec<TrackSize>,
    /// List of grid items: widgets positioned inside the grid.
    items: Vec<GridItem>,
    row_sizes: RefCell<Vec<f64>>,
    column_sizes: RefCell<Vec<f64>>,
}

impl Grid {
    #[composable(uncached)]
    pub fn new() -> Grid {
        Grid {
            columns: vec![],
            rows: vec![],
            items: vec![],
            row_sizes: RefCell::new(vec![]),
            column_sizes: RefCell::new(vec![]),
        }
    }

    pub fn column(mut self, size: GridLength) -> Self {
        self.columns.push(TrackSize {
            min_size: size,
            max_size: size,
        });
        self
    }

    pub fn row(mut self, size: GridLength) -> Self {
        self.rows.push(TrackSize {
            min_size: size,
            max_size: size,
        });
        self
    }

    #[composable(uncached)]
    pub fn item(
        mut self,
        row_span: impl GridSpan,
        column_span: impl GridSpan,
        widget: impl Widget + 'static,
    ) -> Self {
        self.items.push(GridItem {
            row_range: row_span.resolve(self.rows.len()),
            column_range: column_span.resolve(self.columns.len()),
            widget: WidgetPod::new(widget),
        });
        self
    }
}

fn items_in_track(
    items: &[GridItem],
    track_kind: TrackKind,
    track_index: usize,
) -> impl Iterator<Item = &GridItem> {
    items
        .iter()
        .filter(move |item| item.track_span(track_kind).start == track_index)
}

fn size_tracks(
    layout_ctx: &mut LayoutCtx,
    available_space: f64,
    tk: TrackKind,
    tracks: &[TrackSize],
    items: &[GridItem],
    env: &Environment,
) -> Vec<f64> {
    let mut base_size = vec![0.0; tracks.len()];
    let mut growth_limit = vec![0.0; tracks.len()];

    // 11.4. Initialize Track Sizes (https://www.w3.org/TR/css-grid-1/#algo-init)
    for (i, t) in tracks.iter().enumerate() {
        base_size[i] = match t.min_size {
            GridLength::Auto => 0.0,
            GridLength::Fixed(x) => x,
            GridLength::Flex(_) => {
                0.0
                //panic!("flex-size is invalid as a track size minimum")
            }
        };

        growth_limit[i] = match t.max_size {
            GridLength::Fixed(x) => x,
            GridLength::Auto | GridLength::Flex(_) => f64::INFINITY,
        };

        if growth_limit[i] < base_size[i] {
            growth_limit[i] = base_size[i];
        }
    }

    // 11.5 Resolve Intrinsic Track Sizes
    // 2. Size tracks to fit non-spanning items (https://www.w3.org/TR/css-grid-1/#algo-single-span-items)

    for (i, t) in tracks.iter().enumerate() {
        match t.min_size {
            GridLength::Auto => {
                // size = maximum of "natural" sizes
                // filter items whose spans start at the current track
                for item in items_in_track(items, tk, i) {
                    // min-content => hypothetical "minimum size"
                    // max-content => result of layout with no constraints
                    // `width: 100%` => tight constraint
                    // layout first with no constraints to get the preferred size
                    let m =
                        item.widget
                            .layout(layout_ctx, BoxConstraints::tight(Size::zero()), env);
                    base_size[i] = f64::max(base_size[i], track_width(tk, m.size()));
                }
            }
            _ => {}
        }

        match t.max_size {
            GridLength::Auto => {
                for item in items_in_track(items, tk, i) {
                    let m = item
                        .widget
                        .layout(layout_ctx, BoxConstraints::new(.., ..), env);
                    growth_limit[i] = f64::max(base_size[i], track_width(tk, m.size()));
                }
            }
            _ => {}
        }

        if growth_limit[i] < base_size[i] {
            growth_limit[i] = base_size[i];
        }
    }

    // 4. Increase sizes to accommodate spanning items crossing flexible tracks (https://www.w3.org/TR/css-grid-1/#algo-spanning-flex-items)

    // 11.6. Maximize Tracks (https://www.w3.org/TR/css-grid-1/#algo-grow-tracks)
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

    /*// 11.7. Expand Flexible Tracks (https://www.w3.org/TR/css-grid-1/#algo-flex-tracks)
    if free_space > 0.0 {

    }*/

    tracing::trace!(
        "{:?} base_size={:?}, growth_limit={:?}",
        tk,
        base_size,
        growth_limit
    );
    base_size
    // TODO
}

impl Widget for Grid {
    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        // TODO
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        let column_sizes = size_tracks(
            ctx,
            constraints.max_width(),
            TrackKind::Column,
            &self.columns,
            &self.items,
            env,
        );
        let row_sizes = size_tracks(
            ctx,
            constraints.max_height(),
            TrackKind::Row,
            &self.rows,
            &self.items,
            env,
        );

        let column_x: Vec<_> = column_sizes
            .iter()
            .scan(0.0, |s, &x| {
                let px = *s;
                *s += x;
                Some(px)
            })
            .collect();
        let row_y: Vec<_> = row_sizes
            .iter()
            .scan(0.0, |s, &x| {
                let px = *s;
                *s += x;
                Some(px)
            })
            .collect();

        let width: f64 = column_sizes.iter().sum();
        let height: f64 = row_sizes.iter().sum();

        // layout items
        for item in self.items.iter() {
            let w: f64 = column_sizes[item.column_range.clone()].iter().sum();
            let h: f64 = row_sizes[item.row_range.clone()].iter().sum();
            let constraints = BoxConstraints::loose(Size::new(w, h));
            item.widget.layout(ctx, constraints, env);
            let x = column_x[item.column_range.start];
            let y = row_y[item.row_range.start];
            item.widget.set_child_offset(Offset::new(x, y));
        }

        self.row_sizes.replace(row_sizes);
        self.column_sizes.replace(column_sizes);

        Measurements::new(Rect::new(Point::origin(), Size::new(width, height)))
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        use kyute_shell::skia as sk;
        let height = bounds.size.height;
        let width = bounds.size.width;

        let row_sizes = self.row_sizes.borrow();
        let column_sizes = self.column_sizes.borrow();

        let paint = sk::Paint::new(Color::new(1.0, 0.5, 0.2, 1.0).to_skia(), None);

        for x in std::iter::once(0.0)
            .chain(column_sizes.iter().cloned())
            .scan(0.0, |s, x| {
                *s += x;
                Some(*s)
            })
        {
            ctx.canvas.draw_line(
                Point::new(x + 0.5, 0.5).to_skia(),
                Point::new(x + 0.5, height + 0.5).to_skia(),
                &paint,
            );
        }

        for y in std::iter::once(0.0)
            .chain(row_sizes.iter().cloned())
            .scan(0.0, |s, x| {
                *s += x;
                Some(*s)
            })
        {
            ctx.canvas.draw_line(
                Point::new(0.5, y + 0.5).to_skia(),
                Point::new(width + 0.5, y + 0.5).to_skia(),
                &paint,
            );
        }

        for item in self.items.iter() {
            item.widget.paint(ctx, bounds, env);
        }
    }
}
