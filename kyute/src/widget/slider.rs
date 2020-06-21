//! Sliders provide a way to make a value vary linearly between two bounds by dragging a knob along
//! a line.
use crate::event::Event;
use crate::renderer::LineSegment;
use crate::widget::frame::FrameVisual;
use crate::{
    theme, Rect, Size, BoxConstraints, Environment, EventCtx, LayoutCtx, Measurements, PaintCtx, Point,
    TypedWidget, Visual, Widget,
};
use num_traits::{Float, PrimInt};
use std::any::Any;
use crate::style::PaletteIndex;

/// Utility class representing a slider track on which a knob can move.
pub struct SliderTrack {
    track: LineSegment,
    divisions: Option<u32>,
    // in 0..1
    value: f64,
}

impl SliderTrack {
    fn new(track: LineSegment, divisions: Option<u32>, initial_value: f64) -> SliderTrack {
        SliderTrack {
            track,
            divisions,
            value: initial_value,
        }
    }

    /// Returns the current value of the slider.
    fn value(&self) -> f64 {
        self.value
    }

    /// Ignores divisions.
    fn set_value(&mut self, value: f64) {
        self.value = value.min(1.0).max(0.0);
    }

    /// Returns the value that would be set if the cursor was at the given position.
    fn value_from_position(&self, pos: Point) -> f64 {
        /*let hkw = 0.5 * get_knob_width(track_width, divisions, min_knob_width);
        // at the end of the sliders, there are two "dead zones" of width kw / 2 that
        // put the slider all the way left or right
        let x = pos.x.max(hkw).min(track_width-hkw-1.0);*/

        // project the point on the track line
        let v = self.track.end - self.track.start;
        let c = pos - self.track.start;
        let x = v.normalize().dot(c);
        let track_len = v.length();

        if let Some(div) = self.divisions {
            let div = div as f64;
            (div * x / track_len).floor() / div
        } else {
            dbg!(x / track_len)
        }
    }

    /// Returns the position of the knob on the track.
    fn knob_position(&self) -> Point {
        self.track.start + (self.track.end - self.track.start) * self.value
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

pub struct SliderVisual {
    track: SliderTrack,
    min: f64,
    max: f64,
}

impl Default for SliderVisual {
    fn default() -> Self {
        SliderVisual {
            track: SliderTrack::new(
                LineSegment {
                    start: Point::zero(),
                    end: Point::zero(),
                },
                None,
                0.0,
            ),
            min: 0.0,
            max: 0.0,
        }
    }
}

impl SliderVisual {
    fn update_position(&mut self, cursor: Point) {
        let v = self.track.value_from_position(cursor);
        dbg!(v);
        self.track.set_value(v);
    }
}

impl Visual for SliderVisual {
    fn paint(&mut self, ctx: &mut PaintCtx, env: &Environment) {

        let bounds = ctx.bounds();
        let track_y = env.get(theme::SliderTrackY);
        let track_h = env.get(theme::SliderTrackHeight);
        let knob_w = env.get(theme::SliderKnobWidth);
        let knob_h = env.get(theme::SliderKnobHeight);
        let knob_y = env.get(theme::SliderKnobY);

        let track_x_start = self.track.track.start.x;
        let track_x_end = self.track.track.end.x;

        // track bounds
        let track_bounds = Rect::new(
            Point::new(dbg!(track_x_start), track_y - 0.5*track_h),
            Size::new(track_x_end - track_x_start, track_h),
        );

        let kpos = self.track.knob_position();
        let kx = kpos.x.round()+0.5;

        let knob_bounds = Rect::new(
            Point::new(kx-0.5*knob_w, track_y-knob_y),
            Size::new(knob_w, knob_h),
        );

        // track
        ctx.draw_styled_box_in_bounds(
            "slider_track",
            track_bounds,
            PaletteIndex(0));

        ctx.draw_styled_box_in_bounds(
            "slider_knob",
            knob_bounds,
            PaletteIndex(0));
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerOver(_) | Event::PointerOut(_) => {
                ctx.request_redraw();
            }
            Event::PointerDown(p) => {
                self.update_position(p.pointer.position);
                ctx.capture_pointer();
                ctx.request_focus();
                ctx.request_redraw();
            }
            Event::PointerMove(p) => {
                if ctx.is_capturing_pointer() {
                    self.update_position(p.position);
                    ctx.request_redraw();
                }
            }
            Event::PointerUp(p) => {

            }
            _ => {}
        }
    }

    fn hit_test(&mut self, point: Point, bounds: Rect) -> bool {
        unimplemented!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Floating-point sliders.
pub struct Slider<T> {
    value: T,
    min: T,
    max: T,
    divisions: Option<u32>,
}

impl<T: Float> TypedWidget for Slider<T> {
    type Visual = SliderVisual;

    fn key(&self) -> Option<u64> {
        None
    }

    fn layout(
        self,
        context: &mut LayoutCtx,
        previous_visual: Option<Box<SliderVisual>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<SliderVisual>, Measurements) {
        // last position
        let last_value = previous_visual.map(|v| v.track.value);

        let height = env.get(theme::SliderHeight);
        let knob_width = env.get(theme::SliderKnobWidth);
        let knob_height = env.get(theme::SliderKnobHeight);

        let padding = env.get(theme::SliderPadding);

        // fixed height
        let size = Size::new(
            constraints.max_width(),
            constraints.constrain_height(height),
        );

        // position the slider track inside the layout
        let inner_bounds =
            Rect::new(Point::origin(), size).inflate(padding.horizontal(), padding.vertical());

        // calculate knob width
        //let knob_width = get_knob_width(inner_bounds.size.width, self.divisions, min_knob_width);
        // half knob width
        let hkw = 0.5 * knob_width;
        // y-position of the slider track
        let y = 0.5 * size.height;

        // center vertically, add some padding on the sides to account for padding and half-knob size
        let slider_track = LineSegment {
            start: Point::new(inner_bounds.min_x() + hkw, y),
            end: Point::new(inner_bounds.max_x() - hkw, y),
        };

        let visual = SliderVisual {
            track: SliderTrack {
                value: last_value.unwrap_or_default(),
                track: slider_track,
                divisions: self.divisions,
            },
            min: 0.0,
            max: 0.0,
        };

        (
            Box::new(visual),
            Measurements {
                size,
                baseline: None,
            },
        )
    }
}

impl<T: Float> Slider<T> {
    pub fn new(value: T) -> Slider<T> {
        Slider {
            min: T::zero(),
            max: T::one(),
            divisions: None,
            value,
        }
    }

    pub fn min(mut self, min: T) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: T) -> Self {
        self.max = max;
        self
    }

    pub fn divisions(mut self, divisions: u32) -> Self {
        self.divisions = Some(divisions);
        self
    }
}
