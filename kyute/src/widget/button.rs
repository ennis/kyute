use crate::event::Event;
use crate::layout::{Alignment, align_boxes, SideOffsets};
use crate::layout::BoxConstraints;
use crate::layout::Measurements;
use crate::layout::Offset;
use crate::layout::Point;
use crate::layout::Size;
use crate::renderer::{ButtonState};
use crate::visual::Visual;
use crate::widget::BoxedWidget;
use crate::widget::Widget;
use crate::widget::WidgetExt;
use crate::widget::{Text, TypedWidget};
use crate::{Bounds, EventCtx, LayoutCtx, PaintCtx, env, Environment, theme};
use generational_indextree::NodeId;
use std::any::{Any, TypeId};
use euclid::default::SideOffsets2D;
use kyute_shell::drawing::{Color, IntoBrush};

/// Node visual for a button.
pub struct ButtonVisual<A> {
    on_click: Option<A>,
    bg_color: Color,
    border_color: Color,
}

impl<A> Default for ButtonVisual<A> {
    fn default() -> Self {
        ButtonVisual { on_click: None,
            bg_color: Default::default(),
            border_color: Default::default()
        }
    }
}

impl<A: 'static> Visual for ButtonVisual<A> {
    fn paint(&mut self,
             ctx: &mut PaintCtx)
    {
        let bounds = ctx.bounds();
        let button_state = ButtonState {
            disabled: false,
            clicked: false,
            hot: true,
        };

        // draw the button frame
        let bg_brush = self.bg_color.into_brush(ctx);
        let border_brush = self.border_color.into_brush(ctx);
        ctx.fill_rectangle(bounds, &bg_brush);
        let stroke_size = 1.0;
        ctx.draw_rectangle(
            bounds.inflate(-0.5 * stroke_size, -0.5 * stroke_size),
            &border_brush,
            1.0,
        );
    }

    fn hit_test(&mut self, _point: Point, _bounds: Bounds) -> bool {
        false
    }

    fn event(&mut self, ctx: &mut EventCtx, event: &Event) {
        match event {
            Event::PointerDown(p) => {
                eprintln!("BUTTON CLICKED");
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A clickable push button with a text label.
///
/// This widget is influenced by the following style variables:
/// - [`PADDING`](crate::style::PADDING): padding for the label inside the button.
pub struct Button<A> {
    label: BoxedWidget<A>,
    /// Action to emit on button click.
    on_click: Option<A>,
}

impl<A: 'static> Button<A> {
    /// Creates a new button with the given text as the label.
    pub fn new(label: &str) -> Button<A> {
        Button {
            label: Text::new(label).boxed(),
            on_click: None,
        }
    }
}


//-----------------------------------------------------
// Widget implementation
impl<A: 'static> TypedWidget<A> for Button<A> {
    type Visual = ButtonVisual<A>;

    fn layout(
        self,
        context: &mut LayoutCtx<A>,
        previous_visual: Option<Box<ButtonVisual<A>>>,
        constraints: &BoxConstraints,
        env: Environment,
    ) -> (Box<ButtonVisual<A>>, Measurements)
    {
        let on_click = self.on_click;
        let mut visual = previous_visual.unwrap_or_default();

        visual.bg_color = env.get(theme::ButtonBackgroundColor);
        visual.border_color = env.get(theme::ButtonBorderColor);
        visual.on_click = on_click;
        let min_width = env.get(theme::MinButtonWidth);
        let min_height = env.get(theme::MinButtonHeight);

        // measure the label inside
        let padding : SideOffsets = env.get(theme::ButtonLabelPadding);
        let label_constraints = constraints.deflate(&padding);
        let (label_id, label_measurements) =
            context.emit_child(self.label, &label_constraints, env);

        // now measure the button itself
        let mut measurements = Measurements {
            size: label_measurements.size + Size::new(padding.horizontal(), padding.vertical()),
            baseline: None
        };

        // apply minimum size
        measurements.size.width = measurements.size.width.max(min_width);
        measurements.size.height = measurements.size.width.max(min_height);

        // center the label inside the button
        let label_offset = align_boxes(Alignment::CENTER, &mut measurements, label_measurements);
        context.set_child_offset(label_id, label_offset);

        (visual, measurements)
    }
}
