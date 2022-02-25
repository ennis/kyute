//! Sliders provide a way to make a value vary linearly between two bounds by dragging a knob along
//! a line.
use crate::{
    event::PointerEventKind,
    style::{PaintCtxExt, Path},
    theme,
    widget::prelude::*,
    SideOffsets, Signal,
};
use std::cell::Cell;

/// Utility class representing a slider track on which a knob can move.
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

impl Default for SliderTrack {
    fn default() -> Self {
        SliderTrack {
            start: Default::default(),
            end: Default::default(),
        }
    }
}

/*fn draw_slider_knob(
    ctx: &mut PaintCtx,
    size: Size,
    pos: f64,
    divisions: Option<u32>,
    theme: &Theme,
) {
    // half the height
    let min_knob_w = (0.5 * theme.button_metrics.min_height).ceil();
    let knob_w = get_knob_width(size.width, divisions, min_knob_w);

    let off = ((w - knob_w) * pos).ceil();
    let knob = Rect::new(Point::new(off, 0.0), Size::new(knob_w, h));

    // draw the knob rectangle
    let knob_brush = DEFAULT_COLORS.slider_grab.into_brush();
    ctx.fill_rectangle(knob, &knob_brush);
}*/

pub struct Slider {
    id: WidgetId,
    track: Cell<SliderTrack>,
    value: f64,
    value_changed: Signal<f64>,
    min: f64,
    max: f64,
}

impl Slider {
    /// Creates a slider widget.
    ///
    /// Sliders can be used to pick a numeric value in a specified range.
    ///
    /// # Arguments
    /// * `min` - lower bound of the slider range
    /// * `max` - upper bound of the slider range
    /// * `initial` - initial value of the slider.
    #[composable]
    pub fn new(min: f64, max: f64, value: f64) -> Slider {
        Slider {
            id: WidgetId::here(),
            track: Default::default(),
            value,
            value_changed: Signal::new(),
            min,
            max,
        }
    }

    /// Returns the current value, normalized between 0 and 1.
    fn value_norm(&self) -> f64 {
        (self.value - self.min) / (self.max - self.min)
    }

    /// Returns the current value of the slider.
    pub fn current_value(&self) -> f64 {
        self.value
    }

    pub fn value_changed(&self) -> Option<f64> {
        self.value_changed.value()
    }
}

impl Widget for Slider {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerOver | PointerEventKind::PointerOut => {
                    ctx.request_redraw();
                }
                PointerEventKind::PointerDown => {
                    let new_value = self.track.get().value_from_position(p.position, self.min, self.max);
                    self.value_changed.signal(ctx, new_value);
                    ctx.capture_pointer();
                    ctx.request_focus();
                    ctx.request_redraw();
                }
                PointerEventKind::PointerMove => {
                    if ctx.is_capturing_pointer() {
                        let new_value = self.track.get().value_from_position(p.position, self.min, self.max);
                        self.value_changed.signal(ctx, new_value);
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

    fn paint(&self, ctx: &mut PaintCtx, _bounds: Rect, env: &Environment) {
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
        ctx.draw_styled_box(track_bounds, &style, env);

        Path::new("M 0.5 0.5 L 10.5 0.5 L 10.5 5.5 L 5.5 10.5 L 0.5 5.5 Z")
            .fill(theme::keys::CONTROL_COLOR)
            .draw(ctx, knob_bounds, env);
    }
}
