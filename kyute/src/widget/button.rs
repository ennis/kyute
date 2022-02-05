use crate::{
    align_boxes, cache, composable,
    core2::{EventCtx, LayoutCtx, PaintCtx},
    event::PointerEventKind,
    state::Signal,
    widget::Text,
    Alignment, BoxConstraints, Cache, Environment, Event, Key, Measurements, Rect, SideOffsets,
    Size, Widget, WidgetPod,
};
use kyute::state::State;
use tracing::trace;

#[derive(Clone)]
pub struct Button {
    label: WidgetPod<Text>,
    clicked: Signal<()>,
}

// if Button::new returns a Button, then Button must impl Data so that
// WidgetPod::new(widget) can be cached on the widget.
// -> actually, it might not be necessary:
// -> WidgetPod::new:
//      -> get state from the positional cache
//      -> update the button (replace with new instance)
//

impl Button {
    /// Creates a new button with the specified label.
    #[composable]
    pub fn new(label: String) -> Button {
        Button {
            label: WidgetPod::new(Text::new(label)),
            clicked: Signal::new(),
        }
    }

    /// Returns whether this button has been clicked.
    pub fn clicked(&self) -> bool {
        self.clicked.signalled()
    }
}

impl Widget for Button {
    fn debug_name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn event(&self, ctx: &mut EventCtx, event: &mut Event, env: &Environment) {
        match event {
            Event::Pointer(p) => match p.kind {
                PointerEventKind::PointerDown => {
                    //trace!("button clicked");
                    self.clicked.signal(ctx, ());
                    ctx.request_focus();
                    ctx.request_redraw();
                    ctx.set_handled();
                }
                PointerEventKind::PointerOver => {
                    //trace!("button PointerOver");
                    ctx.request_redraw();
                }
                PointerEventKind::PointerOut => {
                    //trace!("button PointerOut");
                    ctx.request_redraw();
                }
                _ => {}
            },
            _ => {}
        }

        if !ctx.handled() {
            self.label.event(ctx, event, env)
        }
    }

    fn layout(
        &self,
        ctx: &mut LayoutCtx,
        constraints: BoxConstraints,
        env: &Environment,
    ) -> Measurements {
        // measure the label inside
        let padding = SideOffsets::new_all_same(4.0);
        let content_constraints = constraints.deflate(padding);

        let label_measurements = self.label.layout(ctx, content_constraints, env);
        let mut measurements = label_measurements;

        //trace!("label_measurements={:?}", label_measurements);

        // add padding on the sides
        measurements.bounds.size += Size::new(padding.horizontal(), padding.vertical());

        // apply minimum size
        measurements.bounds.size.width = measurements.bounds.size.width.max(10.0);
        measurements.bounds.size.height = measurements.bounds.size.height.max(10.0);

        // constrain size
        measurements.bounds.size = constraints.constrain(measurements.bounds.size);

        //trace!("button_measurements={:?}", measurements);

        // center the text inside the button
        let offset = align_boxes(Alignment::CENTER, &mut measurements, label_measurements);

        //trace!("label offset={:?}", offset);
        self.label.set_child_offset(offset);
        measurements
    }

    fn paint(&self, ctx: &mut PaintCtx, bounds: Rect, env: &Environment) {
        use kyute::{style::*, theme::*};

        //tracing::trace!(?bounds, "button paint");

        let background_gradient = linear_gradient()
            .angle(90.0.degrees())
            .stop(BUTTON_BACKGROUND_BOTTOM_COLOR, 0.0)
            .stop(BUTTON_BACKGROUND_TOP_COLOR, 1.0);

        ctx.draw_visual(
            bounds,
            &Rectangle::new()
                .fill(background_gradient.clone())
                .border(
                    Border::new(1.0.dip())
                        .inside(0.0.dip())
                        .brush(BUTTON_BORDER_BOTTOM_COLOR)
                        .opacity(1.0),
                )
                .border(
                    Border::new(1.0.dip()).outside(0.0.dip()).brush(
                        linear_gradient()
                            .angle(90.0.degrees())
                            .stop(WIDGET_OUTER_GROOVE_BOTTOM_COLOR, 0.0)
                            .stop(WIDGET_OUTER_GROOVE_TOP_COLOR, 0.3),
                    ),
                ),
            env,
        );

        self.label.paint(ctx, bounds, env);
    }
}
