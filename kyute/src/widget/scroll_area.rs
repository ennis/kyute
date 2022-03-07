use crate::{
    widget::{
        grid::GridTrackDefinition, prelude::*, Canvas, Color, Container, DragController, Grid, GridLength,
        LayoutInspector, Null, Thumb, UnitExt,
    },
    PointerEventKind,
};
use kyute::style::BoxStyle;
use std::cell::Cell;

/// Derived from slider code.
#[derive(Copy, Clone, Debug)]
struct SliderTrack {
    start: Point,
    end: Point,
}

impl SliderTrack {
    /// Returns the value that would be set if the cursor was at the given position.
    fn value_from_position(&self, pos: Point, min: f64, max: f64) -> f64 {
        /*let hkw = 0.5 * get_knob_width(track_width, divisions, min_knob_width);
        // at the end of the sliders, there are two "dead zones" of width kw / 2 that
        // put the slider all the way left or right
        let x = pos.x.max(hkw).min(track_width-hkw-1.0);*/

        // project the point on the track line
        let v = self.end - self.start;
        let c = pos - self.start;
        let x = v.normalize().dot(c);
        let track_len = v.length();
        (min + (max - min) * x / track_len).clamp(min, max)
    }

    /// Returns the position of the knob on the track.
    fn knob_position(&self, value: f64) -> Point {
        self.start + (self.end - self.start) * value
    }
}

#[derive(Clone)]
pub struct ScrollBar {
    id: WidgetId,
    track_start: Cell<Point>,
    track_end: Cell<Point>,
    knob_position: f64,
    knob_width: f64,
}

impl ScrollBar {
    #[composable]
    pub fn new(knob_width: f64) -> ScrollBar {
        ScrollBar {
            id: WidgetId::here(),
            track_start: Cell::new(Point::zero()),
            track_end: Cell::new(Point::zero()),
            knob_width,
        }
    }

    fn proportional_position(&self, pointer_pos: Point) -> f64 {
        /*let hkw = 0.5 * get_knob_width(track_width, divisions, min_knob_width);
        // at the end of the sliders, there are two "dead zones" of width kw / 2 that
        // put the slider all the way left or right
        let x = pos.x.max(hkw).min(track_width-hkw-1.0);*/

        let v = self.end - self.start;
        let c = pos - self.start;
        let x = v.normalize().dot(c);
        let track_len = v.length();
        (min + (max - min) * x / track_len).clamp(min, max)
    }
}

impl Widget for ScrollBar {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerOver | PointerEventKind::PointerOut => {
                    ctx.request_redraw();
                }
                PointerEventKind::PointerDown => {
                    let new_value = self.track.get().value_from_position(p.position, self.min, self.max);
                    ctx.cache_mut().signal(&self.value_changed, new_value);
                    ctx.capture_pointer();
                    ctx.request_focus();
                    ctx.request_redraw();
                }
                PointerEventKind::PointerMove => {
                    if ctx.is_capturing_pointer() {
                        let new_value = self.track.get().value_from_position(p.position, self.min, self.max);
                        ctx.cache_mut().signal(&self.value_changed, new_value);
                        ctx.request_redraw();
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn layout(&self, _ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let height = env.get(theme::SLIDER_HEIGHT).unwrap();
        let knob_width = env.get(theme::SLIDER_KNOB_WIDTH).unwrap();
        let knob_height = env.get(theme::SLIDER_KNOB_HEIGHT).unwrap();
        let padding = SideOffsets::new_all_same(0.0);

        // fixed height
        let size = Size::new(constraints.max_width(), constraints.constrain_height(height));

        // position the slider track inside the layout
        let inner_bounds = Rect::new(Point::origin(), size).inner_rect(padding);

        // calculate knob width
        //let knob_width = get_knob_width(inner_bounds.size.width, self.divisions, min_knob_width);
        // half knob width
        let hkw = 0.5 * knob_width;
        // y-position of the slider track
        let y = 0.5 * size.height;

        // center vertically, add some padding on the sides to account for padding and half-knob size
        self.track.set(SliderTrack {
            start: Point::new(inner_bounds.min_x() + hkw, y),
            end: Point::new(inner_bounds.max_x() - hkw, y),
        });

        Measurements {
            bounds: size.into(),
            baseline: None,
        }
    }

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, transform: Transform, env: &Environment) {
        /*let background_gradient = LinearGradient::new()
        .angle(90.0.degrees())
        .stop(BUTTON_BACKGROUND_BOTTOM_COLOR, 0.0)
        .stop(BUTTON_BACKGROUND_TOP_COLOR, 1.0);*/

        let track_y = env.get(theme::SLIDER_TRACK_Y).unwrap_or_default();
        let track_h = env.get(theme::SLIDER_TRACK_HEIGHT).unwrap_or_default();
        let knob_w = env.get(theme::SLIDER_KNOB_WIDTH).unwrap_or_default();
        let knob_h = env.get(theme::SLIDER_KNOB_HEIGHT).unwrap_or_default();
        let knob_y = env.get(theme::SLIDER_KNOB_Y).unwrap_or_default();

        let track_x_start = self.track.get().start.x;
        let track_x_end = self.track.get().end.x;

        // track bounds
        let track_bounds = Rect::new(
            Point::new(track_x_start, track_y - 0.5 * track_h),
            Size::new(track_x_end - track_x_start, track_h),
        );

        let kpos = self.track.get().knob_position(self.value_norm());
        let kx = kpos.x.round() + 0.5;

        let knob_bounds = Rect::new(
            Point::new(kx - 0.5 * knob_w, track_y - knob_y),
            Size::new(knob_w, knob_h),
        );

        // track
        let style = env.get(theme::SLIDER_TRACK).unwrap_or_default();
        ctx.draw_styled_box(track_bounds, &style, transform, env);

        Path::new("M 0.5 0.5 L 10.5 0.5 L 10.5 5.5 L 5.5 10.5 L 0.5 5.5 Z")
            .fill(theme::keys::CONTROL_BORDER_COLOR)
            .draw(ctx, knob_bounds, transform, env);
    }
}

#[derive(Clone)]
pub struct ScrollArea {
    inner: Grid,
}

impl ScrollArea {
    #[composable]
    pub fn new(contents: impl Widget + 'static) -> ScrollArea {
        #[state]
        let mut thumb_pos = 0.0;
        #[state]
        let mut tmp_pos = 0.0;

        #[state]
        let mut content_pos: f64 = 0.0;

        let thumb_height = 50.0;

        let thumb = Container::new(Null)
            .fix_size(Size::new(5.0, thumb_height))
            .box_style(BoxStyle::new().radius(2.dip()).fill(Color::from_hex("#FF7F31")));

        let content = LayoutInspector::new(contents);
        let scroll_bar = LayoutInspector::new(Canvas::new());

        // TODO check that content size is finite
        let viewport_height = scroll_bar.size().height;
        let content_height = content.size().height;
        let thumb_height = (scroll_bar_size.height / content_size.height) * scroll_bar_size.height;

        let content_max_pos = content_height - viewport_height;
        let scroll_bar_max_pos = viewport_height - thumb_height;

        // on drag:

        let thumb_pos = (content_pos / content_max_pos) * scroll_bar_max_pos;

        // there are multiple variables at play:
        // - the current scroll position, as a DIP size
        // - the current scroll thumb position (same)
        // - the size of the scroll bar

        // Roughly, we compute the thumb size like so:
        //  Thumb_Size = (Scroll_Bar_Size / Content_Size)

        let mut scroll_bar = DragController::new(Canvas::new().item(0.0, thumb_pos, thumb))
            .on_started(|| tmp_pos = thumb_pos)
            .on_delta(|offset| thumb_pos = tmp_pos + offset.y);

        let mut content_grid = Grid::with_column_definitions([
            GridTrackDefinition::new(GridLength::Flex(1.0)),
            GridTrackDefinition::new(GridLength::Fixed(5.0)),
        ]);
        //content_grid.add_item(0, 0, Canvas::new().add_item(Offset::new(contents));
        content_grid.add_item(0, 1, scroll_bar);

        // canvas not enough for scroll bar:
        // - positioning in the canvas depends on the parent

        // - inner canvas has a fixed size (say, 0-100 px)
        // - then positioning can be absolute

        ScrollArea { inner: content_grid }
    }
}
