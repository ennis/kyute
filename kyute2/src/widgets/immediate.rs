use crate::{
    BoxConstraints, Ctx, Environment, Event, Geometry, HitTestResult, LayoutCtx, PaintCtx, State, Widget, WidgetPtr,
};
use kurbo::{Point, Size};
use scoped_tls::scoped_thread_local;
use std::cell::RefCell;
use tracing::warn;

mod drawing;
mod geometry;
mod interaction;
mod linsys;

use crate::backend::composition::DrawableSurface;

use crate::widgets::immediate::{
    drawing::{DrawCommand, ResolvedDrawCommand},
    geometry::Rect,
    interaction::{InteractRegion, Interaction, ResolvedInteraction},
};
use linsys::{var, LinExpr, VarId};

/// API to use in immediate mode widget functions.
pub mod api {
    pub use super::{
        drawing::*,
        geometry::*,
        interaction::*,
        linsys::{var, LinExpr, VarId},
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ImCtx {
    constraints: BoxConstraints,
    width: VarId,
    height: VarId,
    baseline: VarId,
    child_widgets: RefCell<Vec<ChildWidget>>,
    linear_system: RefCell<linsys::System>,
    draw_commands: RefCell<Vec<DrawCommand>>,
    interactions: RefCell<Vec<Interaction>>,
}

impl ImCtx {
    fn add_draw_command(&self, cmd: DrawCommand) {
        self.draw_commands.borrow_mut().push(cmd);
    }

    fn add_child_widget(&self, child: WidgetPtr, rect: geometry::Rect) {
        self.child_widgets.borrow_mut().push(ChildWidget {
            widget: child,
            layout_rect: rect,
        });
    }

    fn add_interaction(&self, region: InteractRegion, handler: impl FnMut(&mut Ctx, &Event)) {
        self.interactions.borrow_mut().push(Interaction {
            region,
            handler: Box::new(handler),
        });
    }
}

struct ChildWidget {
    widget: WidgetPtr,
    layout_rect: Rect,
}

struct ResolvedChildWidget {
    widget: WidgetPtr,
    layout_rect: kurbo::Rect,
}

scoped_thread_local!(static IMCTX: ImCtx);

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ImmediateWidget<F> {
    builder: F,
    geometry: Geometry,
    child_widgets: Vec<ResolvedChildWidget>,
    draw_commands: Vec<ResolvedDrawCommand>,
    interactions: Vec<ResolvedInteraction>,
}

impl<F> ImmediateWidget<F> {
    pub fn new(builder: F) -> Self {
        ImmediateWidget {
            builder,
            geometry: Default::default(),
            child_widgets: vec![],
            draw_commands: vec![],
            interactions: vec![],
        }
    }
}

impl<F> ImmediateWidget<F>
where
    F: FnMut(&mut Ctx) + 'static,
{
    fn rebuild(&mut self, cx: &mut Ctx, constraints: &BoxConstraints) {
        // set up the immediate context
        let mut linear_system = linsys::System::new();
        let width = linear_system.add_var();
        let height = linear_system.add_var();
        let baseline = linear_system.add_var();
        let imctx = ImCtx {
            constraints: constraints.clone(),
            width,
            height,
            baseline,
            child_widgets: RefCell::new(Vec::new()),
            linear_system: RefCell::new(linear_system),
            draw_commands: RefCell::new(vec![]),
            interactions: RefCell::new(vec![]),
        };

        // run the function that builds the widget with the immediate context
        IMCTX.set(&imctx, || {
            (self.builder)(cx);

            // Resolve the geometry of the widget
            let width_undef = imctx.width.value().is_none();
            let height_undef = imctx.height.value().is_none();
            let baseline_undef = imctx.baseline.value().is_none();

            if width_undef || height_undef || baseline_undef {
                warn!("Could not determine the geometry of the widget:");
                if width_undef {
                    warn!("- could not resolve the width to a known value");
                }
                if height_undef {
                    warn!("- could not resolve the height to a known value");
                }
                if baseline_undef {
                    warn!("- could not resolve the baseline to a known value");
                }
            }

            let width = imctx.width.value().unwrap_or(0.0);
            let height = imctx.height.value().unwrap_or(0.0);
            let baseline = imctx.baseline.value().unwrap_or(0.0);
            self.geometry = Geometry {
                size: Size::new(width, height),
                baseline: Some(baseline),
                ..Default::default()
            };

            // Resolve the draw commands and child widgets
            self.draw_commands = imctx.draw_commands.take().iter().map(|cmd| cmd.resolve()).collect();

            // Resolve the child widgets
            self.child_widgets = imctx
                .child_widgets
                .take()
                .iter()
                .map(|child| ResolvedChildWidget {
                    widget: child.widget.clone(),
                    layout_rect: child.layout_rect.resolve(),
                })
                .collect();

            // Resolve interactions
            self.interactions = imctx
                .interactions
                .take()
                .iter()
                .map(|interaction| interaction.resolve())
                .collect();
        });

        // remount the child widgets
        self.mount(cx);
    }
}

impl<F> Widget for ImmediateWidget<F>
where
    F: FnMut(&mut Ctx) + 'static,
{
    fn mount(&mut self, cx: &mut Ctx) {}

    fn environment(&self) -> Environment {
        Environment::default()
    }

    fn update(&mut self, cx: &mut Ctx) {
        cx.mark_needs_layout();
    }

    fn event(&mut self, cx: &mut Ctx, event: &mut Event) {}

    fn hit_test(&mut self, ctx: &mut HitTestResult, position: Point) -> bool {
        // TODO
        true
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, constraints: &BoxConstraints) -> Geometry {
        self.rebuild(ctx, constraints);
        self.geometry
    }

    fn paint(&mut self, cx: &mut PaintCtx) {
        cx.with_canvas(|canvas| {
            for cmd in self.draw_commands.iter() {
                cmd.draw(canvas);
            }
        });
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn slider_float(
    value: f64,
    min_value: f64,
    max_value: f64,
    value_changed: impl FnMut(&mut Ctx, f64),
) -> impl Widget {
    let mut pressed = State::new(false);
    ImmediateWidget::new(move |cx| {
        use crate::widgets::immediate::api::*;

        let w = max_width();
        let h = max_height().min(20.0);

        //let p = pressed.get_tracked(cx);

        // slider track
        let knob_radius = h / 2.0;
        let trk_start = knob_radius;
        let trk_end = w - knob_radius;
        let trk_y = h / 2.0;
        let trk_rect = rect();
        trk_rect.left.equals(trk_start);
        trk_rect.right.equals(trk_end);
        trk_rect.top.equals(trk_y - 1.0);
        trk_rect.bottom.equals(trk_y + 1.0);

        let knob = circle();
        knob.radius.equals(knob_radius);
        knob.center.equals((lerp(trk_start, trk_end, 0.5), trk_y));

        interact(rect_xywh(0.0, 0.0, w, h), move |cx, event| match event {
            Event::PointerDown(p) => {
                pressed.set(cx, true);
            }
            Event::PointerUp(p) => {
                pressed.set(cx, false);
            }
            Event::PointerMove(p) => {
                if pressed.get() {
                    // issue: we don't have access to the value of the variables here
                }
            }
            _ => {}
        });

        width().equals(max_width());
        height().equals(h);
    })
}
