//! Sliders provide a way to make a value vary linearly between two bounds by dragging a knob along
//! a line.
use crate::{
    align_boxes,
    event::PointerEventKind,
    style::{PaintCtxExt, Path, Style},
    theme,
    widget::prelude::*,
    SideOffsets, Signal,
};
use kyute_common::Color;
use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

/// Utility class representing a slider track on which a knob can move.
#[derive(Copy, Clone, Debug, Default)]
pub struct SliderTrack {
    pub start: Point,
    pub end: Point,
}

impl SliderTrack {
    /// Returns the value that would be set if the cursor was at the given position.
    pub fn value_from_position(&self, pos: Point, min: f64, max: f64) -> f64 {
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
    pub fn knob_position(&self, value: f64) -> Point {
        self.start + (self.end - self.start) * value
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

#[derive(Clone, Default)]
struct SliderLayout {
    track_y: f64,
    track_h: f64,
    knob_w: f64,
    knob_h: f64,
    knob_y: f64,
    value_norm: f64,
    track: SliderTrack,
    track_style: Style,
}

pub struct Slider {
    id: WidgetId,
    track: Cell<SliderTrack>,
    value: f64,
    value_changed: Signal<f64>,
    min: f64,
    max: f64,
    layout: RefCell<SliderLayout>,
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
            layout: RefCell::new(Default::default()),
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

    pub fn on_value_changed(self, f: impl FnOnce(f64)) -> Self {
        self.value_changed.map(f);
        self
    }
}

impl Widget for Slider {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let padding = SideOffsets::new_all_same(0.0);
        let height = env.get(theme::SLIDER_HEIGHT).unwrap();
        let track_y = env.get(theme::SLIDER_TRACK_Y).unwrap_or_default();
        let track_h = env.get(theme::SLIDER_TRACK_HEIGHT).unwrap_or_default();
        let knob_w = env.get(theme::SLIDER_KNOB_WIDTH).unwrap_or_default();
        let knob_h = env.get(theme::SLIDER_KNOB_HEIGHT).unwrap_or_default();
        let knob_y = env.get(theme::SLIDER_KNOB_Y).unwrap_or_default();

        // fixed height
        let size = Size::new(constraints.max_width(), constraints.constrain_height(height));

        // position the slider track inside the layout
        let inner_bounds = Rect::new(Point::origin(), size).inner_rect(padding);

        // calculate knob width
        //let knob_width = get_knob_width(inner_bounds.size.width, self.divisions, min_knob_width);
        // half knob width
        let hkw = 0.5 * knob_w;
        // y-position of the slider track
        let y = 0.5 * size.height;

        // center vertically, add some padding on the sides to account for padding and half-knob size
        if !ctx.speculative {
            self.track.set(SliderTrack {
                start: Point::new(inner_bounds.min_x() + hkw, y),
                end: Point::new(inner_bounds.max_x() - hkw, y),
            });

            self.layout.replace(SliderLayout {
                track_y,
                track_h,
                knob_w,
                knob_h,
                knob_y,
                value_norm: self.value_norm(),
                track: self.track.get(),
                track_style: theme::SLIDER_TRACK.get(env).unwrap(),
            });
        }

        Measurements::from(size)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        if let Event::Pointer(p) = event {
            match p.kind {
                PointerEventKind::PointerOver | PointerEventKind::PointerOut => {
                    //ctx.request_redraw();
                }
                PointerEventKind::PointerDown => {
                    let new_value = self.track.get().value_from_position(p.position, self.min, self.max);
                    self.value_changed.signal(new_value);
                    ctx.capture_pointer();
                    ctx.request_focus();
                }
                PointerEventKind::PointerMove => {
                    if ctx.is_capturing_pointer() {
                        let new_value = self.track.get().value_from_position(p.position, self.min, self.max);
                        self.value_changed.signal(new_value);
                    }
                }
                _ => {}
            }
        }
    }
    fn paint(&self, ctx: &mut PaintCtx) {
        /*let background_gradient = LinearGradient::new()
        .angle(90.0.degrees())
        .stop(BUTTON_BACKGROUND_BOTTOM_COLOR, 0.0)
        .stop(BUTTON_BACKGROUND_TOP_COLOR, 1.0);*/

        /*let track_y = env.get(theme::SLIDER_TRACK_Y).unwrap_or_default();
        let track_h = env.get(theme::SLIDER_TRACK_HEIGHT).unwrap_or_default();
        let knob_w = env.get(theme::SLIDER_KNOB_WIDTH).unwrap_or_default();
        let knob_h = env.get(theme::SLIDER_KNOB_HEIGHT).unwrap_or_default();
        let knob_y = env.get(theme::SLIDER_KNOB_Y).unwrap_or_default();*/

        let layout = self.layout.borrow();

        let track_x_start = layout.track.start.x;
        let track_x_end = layout.track.end.x;

        // track bounds
        let track_bounds = Rect::new(
            Point::new(track_x_start, layout.track_y - 0.5 * layout.track_h),
            Size::new(track_x_end - track_x_start, layout.track_h),
        );

        let kpos = layout.track.knob_position(layout.value_norm);
        let kx = kpos.x.round() + 0.5;

        let knob_bounds = Rect::new(
            Point::new(kx - 0.5 * layout.knob_w, layout.track_y - layout.knob_y),
            Size::new(layout.knob_w, layout.knob_h),
        );

        // track
        ctx.draw_styled_box(track_bounds, &layout.track_style);

        Path::new("M 0.5 0.5 L 10.5 0.5 L 10.5 5.5 L 5.5 10.5 L 0.5 5.5 Z")
            .fill(Color::new(0.0, 0.0, 0.0, 0.6))
            .draw(ctx, knob_bounds);
    }
}

//--------------------------------------------------------------------------------------------------
pub struct SliderBase {
    id: WidgetId,
    track: Cell<SliderTrack>,
    position: f64,
    position_changed: Signal<f64>,
    knob: Arc<WidgetPod>,
    background: Arc<WidgetPod>,
}

impl SliderBase {
    #[composable]
    pub fn new(position: f64, background: impl Widget + 'static, knob: impl Widget + 'static) -> SliderBase {
        SliderBase {
            id: WidgetId::here(),
            track: Default::default(),
            position,
            position_changed: Signal::new(),
            knob: Arc::new(WidgetPod::new(knob)),
            background: Arc::new(WidgetPod::new(background)),
        }
    }

    /// Returns the current position of the slider.
    pub fn current_position(&self) -> f64 {
        self.position
    }

    pub fn position_changed(&self) -> Option<f64> {
        self.position_changed.value()
    }

    pub fn on_position_changed(self, f: impl FnOnce(f64)) -> Self {
        self.position_changed.map(f);
        self
    }
}

impl Widget for SliderBase {
    fn widget_id(&self) -> Option<WidgetId> {
        Some(self.id)
    }

    fn layout(&self, ctx: &mut LayoutCtx, constraints: BoxConstraints, env: &Environment) -> Measurements {
        let knob_measurements = self.knob.layout(ctx, constraints, env);
        let background_measurements = self.background.layout(ctx, constraints, env);

        let width = background_measurements.width();
        let height = knob_measurements.height().max(background_measurements.height());

        let track_y = height * 0.5;
        let bg_offset_y = 0.5 * (height - background_measurements.height());
        let knob_offset_y = 0.5 * (height - knob_measurements.height());

        if !ctx.speculative {
            self.background.set_offset(Offset::new(0.0, bg_offset_y));
        }

        let hkw = 0.5 * knob_measurements.width();
        let track = SliderTrack {
            start: Point::new(hkw, track_y),
            end: Point::new(width - hkw, track_y),
        };
        self.track.set(track);

        if !ctx.speculative {
            self.knob
                .set_offset(Offset::new(track.knob_position(self.position).x - hkw, knob_offset_y));
        }

        let size = Size::new(width, height);
        Measurements::from(size)
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, _env: &Environment) {
        if let Event::Pointer(p) = event {
            match p.kind {
                PointerEventKind::PointerDown => {
                    let new_pos = self.track.get().value_from_position(p.position, 0.0, 1.0);
                    self.position_changed.signal(new_pos);
                    ctx.capture_pointer();
                    ctx.request_focus();
                }
                PointerEventKind::PointerMove => {
                    if ctx.is_capturing_pointer() {
                        let new_pos = self.track.get().value_from_position(p.position, 0.0, 1.0);
                        self.position_changed.signal(new_pos);
                    }
                }
                _ => {}
            }
        }
    }

    fn paint(&self, ctx: &mut PaintCtx) {
        self.background.paint(ctx);
        self.knob.paint(ctx);
    }
}
